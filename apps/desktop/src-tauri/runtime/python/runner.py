#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import inspect
import json
import queue
import sys
import threading
import time
import traceback
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Protocol, TypeAlias, cast

REQUIRED_DRIVER_METHODS = ("connect", "stop", "read")

JSONScalar: TypeAlias = str | int | float | bool | None
JSONValue: TypeAlias = JSONScalar | List["JSONValue"] | Dict[str, "JSONValue"]
DriverConfigValues: TypeAlias = Dict[str, JSONValue]
SensorPayload: TypeAlias = Dict[str, float]
ActuatorPayload: TypeAlias = Dict[str, float]


@dataclass(frozen=True)
class VariableSpec:
    id: str
    name: str
    type: str
    unit: str
    setpoint: float
    pv_min: float
    pv_max: float
    linked_sensor_ids: List[str]


@dataclass(frozen=True)
class IOGroup:
    ids: List[str]
    count: int
    variables: List[VariableSpec]
    variables_by_id: Dict[str, VariableSpec]


@dataclass(frozen=True)
class PlantContext:
    id: str
    name: str
    variables: List[VariableSpec]
    variables_by_id: Dict[str, VariableSpec]
    sensors: IOGroup
    actuators: IOGroup
    setpoints: Dict[str, float]


@dataclass(frozen=True)
class RuntimeTiming:
    owner: str
    clock: str
    strategy: str
    sample_time_ms: int


@dataclass(frozen=True)
class RuntimeSupervision:
    owner: str
    startup_timeout_ms: int
    shutdown_timeout_ms: int


@dataclass(frozen=True)
class RuntimePaths:
    driver_dir: str
    runtime_dir: str
    venv_python_path: str
    runner_path: str


@dataclass(frozen=True)
class RuntimeContext:
    id: str
    timing: RuntimeTiming
    supervision: RuntimeSupervision
    paths: RuntimePaths


@dataclass(frozen=True)
class DriverContext:
    config: DriverConfigValues
    plant: PlantContext
    runtime: RuntimeContext


class DriverProtocol(Protocol):
    def connect(self) -> bool: ...

    def stop(self) -> bool: ...

    def read(self) -> Dict[str, Dict[str, float]]: ...


def emit(msg_type: str, payload: Optional[Dict[str, Any]] = None) -> None:
    envelope: Dict[str, Any] = {"type": msg_type}
    if payload is not None:
        envelope["payload"] = payload
    sys.stdout.write(json.dumps(envelope, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def log_error(message: str) -> None:
    sys.stderr.write(message + "\n")
    sys.stderr.flush()


def expect_dict(raw_value: Any, context: str) -> Dict[str, Any]:
    if not isinstance(raw_value, dict):
        raise RuntimeError(f"{context} deve ser um objeto JSON")
    return cast(Dict[str, Any], raw_value)


def normalize_config(raw_value: Any, context: str) -> DriverConfigValues:
    if raw_value is None:
        return {}
    if not isinstance(raw_value, dict):
        raise RuntimeError(f"{context} deve ser um objeto JSON")
    return {str(key): cast(JSONValue, value) for key, value in raw_value.items()}


def normalize_string(raw_value: Any, context: str) -> str:
    if not isinstance(raw_value, str) or not raw_value.strip():
        raise RuntimeError(f"{context} deve ser uma string não vazia")
    return raw_value.strip()


def normalize_non_negative_int(raw_value: Any, context: str, default: int = 0) -> int:
    if raw_value is None:
        return default
    resolved = int(raw_value)
    if resolved < 0:
        raise RuntimeError(f"{context} não pode ser negativo")
    return resolved


def normalize_positive_int(raw_value: Any, context: str, default: int = 1) -> int:
    resolved = normalize_non_negative_int(raw_value, context, default)
    if resolved <= 0:
        raise RuntimeError(f"{context} deve ser maior que zero")
    return resolved


def normalize_string_list(raw_value: Any, context: str) -> List[str]:
    if raw_value is None:
        return []
    if not isinstance(raw_value, list):
        raise RuntimeError(f"{context} deve ser um array")
    return [str(value) for value in raw_value]


def normalize_float_map(
    raw_value: Any, context: str, allowed_keys: Optional[set[str]] = None
) -> Dict[str, float]:
    if raw_value is None:
        return {}
    if not isinstance(raw_value, dict):
        raise RuntimeError(f"{context} deve ser um objeto JSON")

    normalized: Dict[str, float] = {}
    for key, value in raw_value.items():
        key_str = str(key)
        if allowed_keys is not None and key_str not in allowed_keys:
            continue
        try:
            normalized[key_str] = float(value)
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError(f"{context}.{key_str} deve ser numérico") from exc

    return normalized


def normalize_variable(raw_value: Any, context: str) -> VariableSpec:
    raw = expect_dict(raw_value, context)

    linked_sensor_ids_raw = raw.get("linked_sensor_ids")
    linked_sensor_ids = (
        normalize_string_list(
            linked_sensor_ids_raw,
            f"{context}.linked_sensor_ids",
        )
        if linked_sensor_ids_raw is not None
        else []
    )

    return VariableSpec(
        id=normalize_string(raw.get("id"), f"{context}.id"),
        name=normalize_string(raw.get("name"), f"{context}.name"),
        type=normalize_string(raw.get("type"), f"{context}.type"),
        unit=normalize_string(raw.get("unit"), f"{context}.unit"),
        setpoint=float(raw.get("setpoint", 0.0) or 0.0),
        pv_min=float(raw.get("pv_min", 0.0) or 0.0),
        pv_max=float(raw.get("pv_max", 0.0) or 0.0),
        linked_sensor_ids=linked_sensor_ids,
    )


def normalize_variable_list(raw_value: Any, context: str) -> List[VariableSpec]:
    if raw_value is None:
        return []
    if not isinstance(raw_value, list):
        raise RuntimeError(f"{context} deve ser um array")

    return [
        normalize_variable(item, f"{context}[{index}]")
        for index, item in enumerate(raw_value)
    ]


def normalize_variable_map(raw_value: Any, context: str) -> Dict[str, VariableSpec]:
    if raw_value is None:
        return {}
    if not isinstance(raw_value, dict):
        raise RuntimeError(f"{context} deve ser um objeto JSON")

    normalized: Dict[str, VariableSpec] = {}
    for key, value in raw_value.items():
        variable = normalize_variable(value, f"{context}.{key}")
        normalized[variable.id] = variable
    return normalized


def build_variable_map(variables: List[VariableSpec]) -> Dict[str, VariableSpec]:
    return {variable.id: variable for variable in variables}


def normalize_io_group(raw_value: Any, context: str) -> IOGroup:
    raw = expect_dict(raw_value, context)

    variables = normalize_variable_list(raw.get("variables"), f"{context}.variables")
    variables_by_id = normalize_variable_map(
        raw.get("variables_by_id"), f"{context}.variables_by_id"
    )
    if not variables_by_id:
        variables_by_id = build_variable_map(variables)
    if not variables:
        variables = list(variables_by_id.values())

    ids = normalize_string_list(raw.get("ids"), f"{context}.ids")
    if not ids:
        ids = [variable.id for variable in variables]

    count = normalize_non_negative_int(raw.get("count"), f"{context}.count", len(ids))

    return IOGroup(
        ids=ids,
        count=count,
        variables=variables,
        variables_by_id=variables_by_id,
    )


def normalize_plant_context(raw_value: Any) -> PlantContext:
    raw = expect_dict(raw_value, "bootstrap.plant")

    variables = normalize_variable_list(
        raw.get("variables"), "bootstrap.plant.variables"
    )
    variables_by_id = normalize_variable_map(
        raw.get("variables_by_id"), "bootstrap.plant.variables_by_id"
    )
    if not variables_by_id:
        variables_by_id = build_variable_map(variables)
    if not variables:
        variables = list(variables_by_id.values())

    return PlantContext(
        id=normalize_string(raw.get("id"), "bootstrap.plant.id"),
        name=normalize_string(raw.get("name"), "bootstrap.plant.name"),
        variables=variables,
        variables_by_id=variables_by_id,
        sensors=normalize_io_group(raw.get("sensors"), "bootstrap.plant.sensors"),
        actuators=normalize_io_group(raw.get("actuators"), "bootstrap.plant.actuators"),
        setpoints=normalize_float_map(
            raw.get("setpoints"), "bootstrap.plant.setpoints"
        ),
    )


def normalize_runtime_context(raw_value: Any) -> RuntimeContext:
    raw = expect_dict(raw_value, "bootstrap.runtime")
    timing_raw = expect_dict(raw.get("timing"), "bootstrap.runtime.timing")
    supervision_raw = expect_dict(
        raw.get("supervision"), "bootstrap.runtime.supervision"
    )
    paths_raw = expect_dict(raw.get("paths"), "bootstrap.runtime.paths")

    return RuntimeContext(
        id=normalize_string(raw.get("id"), "bootstrap.runtime.id"),
        timing=RuntimeTiming(
            owner=normalize_string(
                timing_raw.get("owner"), "bootstrap.runtime.timing.owner"
            ),
            clock=normalize_string(
                timing_raw.get("clock"), "bootstrap.runtime.timing.clock"
            ),
            strategy=normalize_string(
                timing_raw.get("strategy"), "bootstrap.runtime.timing.strategy"
            ),
            sample_time_ms=normalize_positive_int(
                timing_raw.get("sample_time_ms"),
                "bootstrap.runtime.timing.sample_time_ms",
                100,
            ),
        ),
        supervision=RuntimeSupervision(
            owner=normalize_string(
                supervision_raw.get("owner"),
                "bootstrap.runtime.supervision.owner",
            ),
            startup_timeout_ms=normalize_positive_int(
                supervision_raw.get("startup_timeout_ms"),
                "bootstrap.runtime.supervision.startup_timeout_ms",
                1000,
            ),
            shutdown_timeout_ms=normalize_positive_int(
                supervision_raw.get("shutdown_timeout_ms"),
                "bootstrap.runtime.supervision.shutdown_timeout_ms",
                1000,
            ),
        ),
        paths=RuntimePaths(
            driver_dir=normalize_string(
                paths_raw.get("driver_dir"), "bootstrap.runtime.paths.driver_dir"
            ),
            runtime_dir=normalize_string(
                paths_raw.get("runtime_dir"), "bootstrap.runtime.paths.runtime_dir"
            ),
            venv_python_path=normalize_string(
                paths_raw.get("venv_python_path"),
                "bootstrap.runtime.paths.venv_python_path",
            ),
            runner_path=normalize_string(
                paths_raw.get("runner_path"), "bootstrap.runtime.paths.runner_path"
            ),
        ),
    )


def normalize_bootstrap(raw_value: Any) -> tuple[str, DriverContext]:
    raw = expect_dict(raw_value, "bootstrap")
    driver_raw = expect_dict(raw.get("driver"), "bootstrap.driver")
    plant_context = normalize_plant_context(raw.get("plant"))
    runtime_context = normalize_runtime_context(raw.get("runtime"))

    driver_class_name = normalize_string(
        driver_raw.get("class_name"),
        "bootstrap.driver.class_name",
    )
    driver_config = normalize_config(
        driver_raw.get("config"), "bootstrap.driver.config"
    )

    return driver_class_name, DriverContext(
        config=driver_config,
        plant=plant_context,
        runtime=runtime_context,
    )


def load_driver_class(driver_dir: Path, expected_class_name: str) -> type[Any]:
    driver_main = driver_dir / "main.py"
    if not driver_main.exists():
        raise RuntimeError(f"main.py não encontrado em '{driver_main}'")

    spec = importlib.util.spec_from_file_location("driver_module", str(driver_main))
    if spec is None or spec.loader is None:
        raise RuntimeError("Falha ao criar spec do módulo do driver")

    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    candidate = getattr(module, expected_class_name, None)
    if candidate is None or not inspect.isclass(candidate):
        raise RuntimeError(
            f"Classe de driver '{expected_class_name}' não encontrada em main.py"
        )
    if candidate.__module__ != module.__name__:
        raise RuntimeError(
            f"Classe de driver '{expected_class_name}' precisa ser definida em main.py"
        )

    missing = [
        method
        for method in REQUIRED_DRIVER_METHODS
        if not callable(getattr(candidate, method, None))
    ]
    if missing:
        raise RuntimeError(
            f"Classe de driver '{expected_class_name}' inválida. Métodos ausentes: {', '.join(missing)}"
        )

    return candidate


def instantiate_driver(driver_cls: type[Any], context: DriverContext) -> DriverProtocol:
    try:
        instance = driver_cls(context)
    except TypeError as exc:
        raise RuntimeError(
            "Construtor do driver deve seguir o contrato __init__(self, context)"
        ) from exc

    return cast(DriverProtocol, instance)


def coerce_method_status(method_name: str, result: Any) -> bool:
    if not isinstance(result, bool):
        raise RuntimeError(
            f"Método '{method_name}' deve retornar bool, recebeu {type(result).__name__}"
        )
    return result


def spawn_command_reader(command_queue: "queue.Queue[Dict[str, Any]]") -> None:
    def _reader() -> None:
        for raw_line in sys.stdin:
            line = raw_line.strip()
            if not line:
                continue
            try:
                payload = json.loads(line)
            except Exception as exc:  # noqa: BLE001
                emit("error", {"message": f"Comando JSON inválido: {exc}"})
                continue

            if not isinstance(payload, dict):
                emit("error", {"message": "Comando recebido deve ser um objeto JSON"})
                continue

            command_queue.put(cast(Dict[str, Any], payload))

    thread = threading.Thread(target=_reader, daemon=True, name="stdin-command-reader")
    thread.start()


def normalize_read_snapshot(
    raw_value: Any,
    context: DriverContext,
) -> tuple[SensorPayload, ActuatorPayload]:
    if raw_value is None:
        return {}, {}
    if not isinstance(raw_value, dict):
        raise RuntimeError(
            "read() deve retornar um objeto JSON no formato {'sensors': {...}, 'actuators': {...}}"
        )

    sensor_ids = set(context.plant.sensors.ids)
    actuator_ids = set(context.plant.actuators.ids)

    sensors = normalize_float_map(
        raw_value.get("sensors"), "read().sensors", sensor_ids
    )
    actuators = normalize_float_map(
        raw_value.get("actuators"), "read().actuators", actuator_ids
    )
    return sensors, actuators


def run() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runtime-dir", required=True)
    parser.add_argument("--driver-dir", required=True)
    parser.add_argument("--bootstrap", required=True)
    args = parser.parse_args()

    runtime_dir = Path(args.runtime_dir)
    driver_dir = Path(args.driver_dir)
    bootstrap_path = Path(args.bootstrap)

    if not bootstrap_path.exists():
        emit(
            "error", {"message": f"bootstrap.json não encontrado em '{bootstrap_path}'"}
        )
        return 1

    with bootstrap_path.open("r", encoding="utf-8") as handle:
        driver_class_name, context = normalize_bootstrap(json.load(handle))

    runtime_id = context.runtime.id
    plant_id = context.plant.id
    sample_time_ms = context.runtime.timing.sample_time_ms

    command_queue: "queue.Queue[Dict[str, Any]]" = queue.Queue()
    spawn_command_reader(command_queue)

    emit(
        "ready",
        {
            "runtime_id": runtime_id,
            "plant_id": plant_id,
            "driver": str(driver_dir.name),
            "runtime_dir": str(runtime_dir),
        },
    )

    initialized = False
    running = False
    paused = False
    should_exit = False
    cycle_id = 0
    runtime_started_at: Optional[float] = None
    last_cycle_started_at: Optional[float] = None
    next_cycle_deadline: Optional[float] = None
    paused_started_at: Optional[float] = None
    paused_duration_s = 0.0
    driver_instance: Optional[DriverProtocol] = None

    while not should_exit:
        while True:
            try:
                command = command_queue.get_nowait()
            except queue.Empty:
                break

            msg_type = str(command.get("type", "")).strip()
            payload = command.get("payload")

            if msg_type == "init":
                try:
                    driver_class_name, context = normalize_bootstrap(payload)
                    runtime_id = context.runtime.id
                    plant_id = context.plant.id
                    sample_time_ms = context.runtime.timing.sample_time_ms
                    initialized = True
                except Exception as exc:  # noqa: BLE001
                    emit(
                        "error", {"message": f"Falha ao aplicar bootstrap init: {exc}"}
                    )
                    should_exit = True
                    break
                continue

            if msg_type == "start":
                if not initialized:
                    emit(
                        "error", {"message": "Runtime recebeu 'start' antes de 'init'"}
                    )
                    continue

                if driver_instance is None:
                    try:
                        driver_cls = load_driver_class(driver_dir, driver_class_name)
                        driver_instance = instantiate_driver(driver_cls, context)
                        connected = coerce_method_status(
                            "connect", driver_instance.connect()
                        )
                        if not connected:
                            raise RuntimeError("Driver retornou False em connect()")
                        runtime_started_at = time.monotonic()
                        emit(
                            "connected",
                            {"runtime_id": runtime_id, "plant_id": plant_id},
                        )
                    except Exception as exc:  # noqa: BLE001
                        log_error(traceback.format_exc())
                        emit(
                            "error", {"message": f"Falha ao inicializar driver: {exc}"}
                        )
                        should_exit = True
                        break

                running = True
                paused = False
                now = time.monotonic()
                if runtime_started_at is None:
                    runtime_started_at = now
                next_cycle_deadline = now
                continue

            if msg_type == "pause":
                if not paused:
                    paused_started_at = time.monotonic()
                paused = True
                next_cycle_deadline = None
                last_cycle_started_at = None
                continue

            if msg_type == "resume":
                if paused_started_at is not None:
                    paused_duration_s += max(0.0, time.monotonic() - paused_started_at)
                    paused_started_at = None
                paused = False
                next_cycle_deadline = time.monotonic() + (sample_time_ms / 1000.0)
                last_cycle_started_at = None
                continue

            if msg_type in ("stop", "shutdown"):
                should_exit = True
                running = False
                break

            if msg_type == "write_outputs":
                # TODO
                continue

        if should_exit:
            break

        if not running or paused:
            time.sleep(0.01)
            continue

        if next_cycle_deadline is None:
            next_cycle_deadline = time.monotonic()

        now = time.monotonic()
        if now < next_cycle_deadline:
            time.sleep(next_cycle_deadline - now)

        cycle_started_at = time.monotonic()
        cycle_id += 1
        read_started_at = cycle_started_at

        sensors: SensorPayload = {}
        actuators: ActuatorPayload = {}
        try:
            if driver_instance is not None:
                sensors, actuators = normalize_read_snapshot(
                    driver_instance.read(), context
                )
        except Exception as exc:  # noqa: BLE001
            log_error(traceback.format_exc())
            emit("warning", {"message": f"Falha em leitura de driver: {exc}"})

        read_duration_ms = (time.monotonic() - read_started_at) * 1000.0
        cycle_finished_at = time.monotonic()
        cycle_duration_ms = (cycle_finished_at - cycle_started_at) * 1000.0

        if last_cycle_started_at is None:
            effective_dt_ms = float(sample_time_ms)
        else:
            effective_dt_ms = max(
                0.0, (cycle_started_at - last_cycle_started_at) * 1000.0
            )

        sample_step = sample_time_ms / 1000.0
        planned_next_deadline = next_cycle_deadline + sample_step
        late_by_ms = max(0.0, (cycle_finished_at - planned_next_deadline) * 1000.0)
        cycle_late = late_by_ms > 0.0

        emit(
            "telemetry",
            {
                "timestamp": time.time(),
                "cycle_id": cycle_id,
                "configured_sample_time_ms": sample_time_ms,
                "effective_dt_ms": effective_dt_ms,
                "cycle_duration_ms": cycle_duration_ms,
                "read_duration_ms": read_duration_ms,
                "cycle_late": cycle_late,
                "phase": "publish_telemetry",
                "uptime_s": max(
                    0.0,
                    time.monotonic()
                    - (runtime_started_at or cycle_started_at)
                    - paused_duration_s,
                ),
                "sensors": sensors,
                "actuators": actuators,
                "setpoints": context.plant.setpoints,
            },
        )

        if cycle_late:
            emit(
                "cycle_overrun",
                {
                    "cycle_id": cycle_id,
                    "configured_sample_time_ms": sample_time_ms,
                    "cycle_duration_ms": cycle_duration_ms,
                    "late_by_ms": late_by_ms,
                    "phase": "read_inputs",
                },
            )

        next_cycle_deadline = planned_next_deadline
        while next_cycle_deadline < time.monotonic():
            next_cycle_deadline += sample_step

        last_cycle_started_at = cycle_started_at

    try:
        if driver_instance is not None:
            stopped = coerce_method_status("stop", driver_instance.stop())
            if not stopped:
                emit("warning", {"message": "Driver retornou False em stop()"})
    except Exception as exc:  # noqa: BLE001
        log_error(f"Falha ao finalizar driver: {exc}")

    emit("stopped", {"runtime_id": runtime_id, "plant_id": plant_id})
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(run())
    except Exception as exc:  # noqa: BLE001
        log_error(traceback.format_exc())
        emit("error", {"message": f"Runner Python falhou: {exc}"})
        raise
