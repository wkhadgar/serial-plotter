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
import types
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Protocol, TypeAlias, TypedDict, cast

REQUIRED_DRIVER_METHODS = ("connect", "stop", "read")
OPTIONAL_DRIVER_METHODS = ("reconnect", "send")

JSONScalar: TypeAlias = str | int | float | bool | None
JSONValue: TypeAlias = JSONScalar | List["JSONValue"] | Dict[str, "JSONValue"]
DriverParams: TypeAlias = Dict[str, JSONValue]
SensorPayload: TypeAlias = Dict[str, float]
ActuatorPayload: TypeAlias = Dict[str, float]
SetpointPayload: TypeAlias = Dict[str, float]


class PlantVariablePayload(TypedDict, total=False):
    id: str
    name: str
    type: str
    unit: str
    setpoint: float
    pv_min: float
    pv_max: float
    linked_sensor_ids: List[str]


class BootstrapPayload(TypedDict, total=False):
    runtime_id: str
    plant_id: str
    plant_name: str
    sample_time_ms: int
    driver_class_name: str
    driver_config: DriverParams
    variables: List[PlantVariablePayload]
    sensors: List[str]
    actuators: List[str]
    setpoints: SetpointPayload
    driver_dir: str
    runtime_dir: str
    venv_python_path: str
    runner_path: str


class DriverRuntimeContext(TypedDict):
    runtime_id: str
    plant_id: str
    plant_name: str
    sample_time_ms: int
    variables: List[PlantVariablePayload]
    sensors: List[str]
    actuators: List[str]
    setpoints: SetpointPayload
    driver_dir: str
    runtime_dir: str
    venv_python_path: str
    runner_path: str


class DriverProtocol(Protocol):
    config: "DriverConfig"
    params: DriverParams
    runtime: DriverRuntimeContext

    def connect(self) -> bool: ...

    def reconnect(self) -> bool: ...

    def stop(self) -> bool: ...

    def read(self) -> SensorPayload: ...

    def send(self, outputs: Optional[ActuatorPayload] = None) -> bool: ...


def empty_runtime_context() -> DriverRuntimeContext:
    return {
        "runtime_id": "",
        "plant_id": "",
        "plant_name": "",
        "sample_time_ms": 100,
        "variables": [],
        "sensors": [],
        "actuators": [],
        "setpoints": {},
        "driver_dir": "",
        "runtime_dir": "",
        "venv_python_path": "",
        "runner_path": "",
    }


class DriverConfig(dict[str, JSONValue]):
    """Config dict-like com metadados do runtime anexados pelo runner."""

    def __init__(
        self,
        params: Optional[Mapping[str, JSONValue]] = None,
        *,
        runtime: Optional[DriverRuntimeContext] = None,
    ) -> None:
        normalized_params = dict(params or {})
        super().__init__(normalized_params)
        self.params: DriverParams = normalized_params
        self.runtime: DriverRuntimeContext = runtime or empty_runtime_context()

    @property
    def variables(self) -> List[PlantVariablePayload]:
        return self.runtime["variables"]

    @property
    def sensors(self) -> List[str]:
        return self.runtime["sensors"]

    @property
    def actuators(self) -> List[str]:
        return self.runtime["actuators"]

    @property
    def setpoints(self) -> SetpointPayload:
        return self.runtime["setpoints"]


class MCUDriver:
    """Classe base compatível com o template gerado no editor de código."""

    def __init__(self, **kwargs: JSONValue) -> None:
        self.config = DriverConfig(kwargs)
        self.params = self.config.params
        self.runtime = self.config.runtime

    def connect(self) -> bool:
        return True

    def reconnect(self) -> bool:
        return True

    def stop(self) -> bool:
        return True

    def read(self) -> SensorPayload:
        return {}

    def send(self, outputs: Optional[ActuatorPayload] = None) -> bool:
        _ = outputs
        return True


def install_senamby_module() -> None:
    """Expõe módulo `senamby` para o driver importar `MCUDriver`."""
    module_name = "senamby"
    module = sys.modules.get(module_name)
    if module is None:
        module = types.ModuleType(module_name)
        sys.modules[module_name] = module

    setattr(module, "MCUDriver", MCUDriver)


def emit(msg_type: str, payload: Optional[Dict[str, Any]] = None) -> None:
    envelope: Dict[str, Any] = {"type": msg_type}
    if payload is not None:
        envelope["payload"] = payload
    sys.stdout.write(json.dumps(envelope, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def log_error(message: str) -> None:
    sys.stderr.write(message + "\n")
    sys.stderr.flush()


def load_driver_class(driver_dir: Path, expected_class_name: Optional[str]):
    install_senamby_module()

    driver_main = driver_dir / "main.py"
    if not driver_main.exists():
        raise RuntimeError(f"main.py não encontrado em '{driver_main}'")

    spec = importlib.util.spec_from_file_location("driver_module", str(driver_main))
    if spec is None or spec.loader is None:
        raise RuntimeError("Falha ao criar spec do módulo do driver")

    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    if expected_class_name and hasattr(module, expected_class_name):
        candidate = getattr(module, expected_class_name)
        if inspect.isclass(candidate):
            return validate_driver_class(candidate, expected_class_name)

    if hasattr(module, "Driver") and inspect.isclass(module.Driver):
        return validate_driver_class(module.Driver, "Driver")

    for _, cls in inspect.getmembers(module, inspect.isclass):
        if cls.__module__ != module.__name__:
            continue
        if issubclass(cls, MCUDriver) and cls is not MCUDriver:
            return validate_driver_class(cls, cls.__name__)
        if all(hasattr(cls, method) for method in REQUIRED_DRIVER_METHODS):
            return validate_driver_class(cls, cls.__name__)

    raise RuntimeError("Nenhuma classe de driver compatível foi encontrada em main.py")


def validate_driver_class(driver_cls: Any, class_label: str):
    missing = [
        method
        for method in REQUIRED_DRIVER_METHODS
        if not callable(getattr(driver_cls, method, None))
    ]
    if missing:
        raise RuntimeError(
            f"Classe de driver '{class_label}' inválida. Métodos ausentes: {', '.join(missing)}"
        )
    return driver_cls


def normalize_driver_params(raw_params: Any) -> DriverParams:
    if not isinstance(raw_params, dict):
        return {}
    return {str(key): cast(JSONValue, value) for key, value in raw_params.items()}


def normalize_variables(raw_variables: Any) -> List[PlantVariablePayload]:
    if not isinstance(raw_variables, list):
        return []

    normalized: List[PlantVariablePayload] = []
    for item in raw_variables:
        if not isinstance(item, dict):
            continue

        variable: PlantVariablePayload = {}
        for key in ("id", "name", "type", "unit"):
            value = item.get(key)
            if isinstance(value, str):
                variable[key] = value

        for key in ("setpoint", "pv_min", "pv_max"):
            value = item.get(key)
            if isinstance(value, (int, float)):
                variable[key] = float(value)

        linked_sensor_ids = item.get("linked_sensor_ids")
        if isinstance(linked_sensor_ids, list):
            variable["linked_sensor_ids"] = [str(value) for value in linked_sensor_ids]

        normalized.append(variable)

    return normalized


def normalize_string_list(raw_values: Any) -> List[str]:
    if not isinstance(raw_values, list):
        return []
    return [str(value) for value in raw_values]


def build_runtime_context(bootstrap: BootstrapPayload) -> DriverRuntimeContext:
    raw_setpoints = bootstrap.get("setpoints")
    normalized_setpoints = normalize_sensors(raw_setpoints)

    return {
        "runtime_id": str(bootstrap.get("runtime_id", "")),
        "plant_id": str(bootstrap.get("plant_id", "")),
        "plant_name": str(bootstrap.get("plant_name", "")),
        "sample_time_ms": int(bootstrap.get("sample_time_ms", 100) or 100),
        "variables": normalize_variables(bootstrap.get("variables")),
        "sensors": normalize_string_list(bootstrap.get("sensors")),
        "actuators": normalize_string_list(bootstrap.get("actuators")),
        "setpoints": normalized_setpoints,
        "driver_dir": str(bootstrap.get("driver_dir", "")),
        "runtime_dir": str(bootstrap.get("runtime_dir", "")),
        "venv_python_path": str(bootstrap.get("venv_python_path", "")),
        "runner_path": str(bootstrap.get("runner_path", "")),
    }


def attach_driver_config(driver_instance: Any, driver_config: DriverConfig) -> DriverProtocol:
    existing_config = getattr(driver_instance, "config", None)

    if isinstance(existing_config, DriverConfig):
        bound_config = DriverConfig(existing_config, runtime=driver_config.runtime)
    elif isinstance(existing_config, dict):
        bound_config = DriverConfig(existing_config, runtime=driver_config.runtime)
    else:
        bound_config = DriverConfig(driver_config.params, runtime=driver_config.runtime)

    setattr(driver_instance, "config", bound_config)
    setattr(driver_instance, "params", bound_config.params)
    setattr(driver_instance, "runtime", bound_config.runtime)
    return cast(DriverProtocol, driver_instance)


def instantiate_driver(driver_cls: Any, driver_config: DriverConfig) -> DriverProtocol:
    try:
        instance = driver_cls(**driver_config.params)
    except TypeError:
        try:
            # Compatibilidade com drivers legados que aceitam dicionário único.
            instance = driver_cls(driver_config.params)
        except TypeError:
            # Compatibilidade com drivers que não recebem argumentos.
            instance = driver_cls()

    ensure_optional_methods(instance)
    return attach_driver_config(instance, driver_config)


def ensure_optional_methods(driver_instance: Any) -> None:
    for method in OPTIONAL_DRIVER_METHODS:
        if callable(getattr(driver_instance, method, None)):
            continue

        if method == "send":
            setattr(driver_instance, "send", lambda *outs: True)
        elif method == "reconnect":
            setattr(driver_instance, "reconnect", lambda: True)


def coerce_method_status(method_name: str, result: Any, *, default: bool = True) -> bool:
    if result is None:
        return default
    if isinstance(result, bool):
        return result
    raise TypeError(
        f"Método '{method_name}' deve retornar bool, recebeu {type(result).__name__}"
    )


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
            command_queue.put(payload)

    thread = threading.Thread(target=_reader, daemon=True, name="stdin-command-reader")
    thread.start()


def normalize_sensors(raw_value: Any) -> Dict[str, float]:
    if raw_value is None:
        return {}
    if isinstance(raw_value, dict):
        normalized: Dict[str, float] = {}
        for key, value in raw_value.items():
            try:
                normalized[str(key)] = float(value)
            except Exception:  # noqa: BLE001
                continue
        return normalized
    return {}


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
        emit("error", {"message": f"bootstrap.json não encontrado em '{bootstrap_path}'"})
        return 1

    with bootstrap_path.open("r", encoding="utf-8") as fp:
        bootstrap = cast(BootstrapPayload, json.load(fp))

    runtime_id = str(bootstrap.get("runtime_id", ""))
    plant_id = str(bootstrap.get("plant_id", ""))
    driver_class_name = bootstrap.get("driver_class_name")
    if isinstance(driver_class_name, str):
        driver_class_name = driver_class_name.strip() or None
    else:
        driver_class_name = None
    sample_time_ms = int(bootstrap.get("sample_time_ms", 100))
    if sample_time_ms <= 0:
        sample_time_ms = 100

    driver_params = normalize_driver_params(bootstrap.get("driver_config"))
    runtime_context = build_runtime_context(bootstrap)
    driver_config = DriverConfig(driver_params, runtime=runtime_context)
    setpoints = normalize_sensors(bootstrap.get("setpoints"))

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
    last_cycle_started_at: Optional[float] = None
    next_deadline: Optional[float] = None
    driver_instance: Optional[Any] = None

    while not should_exit:
        while True:
            try:
                cmd = command_queue.get_nowait()
            except queue.Empty:
                break

            msg_type = str(cmd.get("type", "")).strip()
            payload = cmd.get("payload") or {}

            if msg_type == "init":
                initialized = True
                if isinstance(payload, dict):
                    sample_time_ms = int(payload.get("sample_time_ms", sample_time_ms) or sample_time_ms)
                    if sample_time_ms <= 0:
                        sample_time_ms = 100
                    driver_params = normalize_driver_params(payload.get("driver_config"))
                    driver_config = DriverConfig(driver_params or driver_config.params, runtime=runtime_context)
                    runtime_context["sample_time_ms"] = sample_time_ms

                    if "variables" in payload:
                        runtime_context["variables"] = normalize_variables(payload.get("variables"))
                    if "sensors" in payload:
                        runtime_context["sensors"] = normalize_string_list(payload.get("sensors"))
                    if "actuators" in payload:
                        runtime_context["actuators"] = normalize_string_list(payload.get("actuators"))

                    setpoints = normalize_sensors(payload.get("setpoints")) or setpoints
                    runtime_context["setpoints"] = dict(setpoints)
                continue

            if msg_type == "start":
                if not initialized:
                    emit("error", {"message": "Runtime recebeu 'start' antes de 'init'"})
                    continue
                if driver_instance is None:
                    try:
                        driver_cls = load_driver_class(driver_dir, driver_class_name)
                        driver_instance = instantiate_driver(driver_cls, driver_config)
                        if hasattr(driver_instance, "connect"):
                            connected = coerce_method_status(
                                "connect",
                                driver_instance.connect(),
                            )
                            if not connected:
                                raise RuntimeError("Driver retornou False em connect()")
                        emit("connected", {"runtime_id": runtime_id, "plant_id": plant_id})
                    except Exception as exc:  # noqa: BLE001
                        log_error(traceback.format_exc())
                        emit("error", {"message": f"Falha ao inicializar driver: {exc}"})
                        should_exit = True
                        break

                running = True
                paused = False
                now = time.monotonic()
                next_deadline = now
                continue

            if msg_type == "pause":
                paused = True
                continue

            if msg_type == "resume":
                paused = False
                if next_deadline is None:
                    next_deadline = time.monotonic()
                continue

            if msg_type in ("stop", "shutdown"):
                should_exit = True
                running = False
                break

            if msg_type == "write_outputs":
                # Escopo atual: leitura apenas.
                continue

        if should_exit:
            break

        if not running or paused:
            time.sleep(0.01)
            continue

        if next_deadline is None:
            next_deadline = time.monotonic()

        now = time.monotonic()
        if now < next_deadline:
            time.sleep(next_deadline - now)

        cycle_started_at = time.monotonic()
        cycle_id += 1
        read_started_at = cycle_started_at

        sensors: Dict[str, float] = {}
        try:
            if driver_instance is not None and hasattr(driver_instance, "read"):
                sensors = normalize_sensors(driver_instance.read())
        except Exception as exc:  # noqa: BLE001
            log_error(traceback.format_exc())
            emit("warning", {"message": f"Falha em leitura de driver: {exc}"})

        read_duration_ms = (time.monotonic() - read_started_at) * 1000.0
        cycle_finished_at = time.monotonic()
        cycle_duration_ms = (cycle_finished_at - cycle_started_at) * 1000.0

        if last_cycle_started_at is None:
            effective_dt_ms = float(sample_time_ms)
        else:
            effective_dt_ms = max(0.0, (cycle_started_at - last_cycle_started_at) * 1000.0)

        late_by_ms = (cycle_finished_at - next_deadline) * 1000.0
        cycle_late = late_by_ms > 0

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
                "uptime_s": max(0.0, time.monotonic() - (last_cycle_started_at or cycle_started_at)),
                "sensors": sensors,
                "actuators": {},
                "setpoints": setpoints,
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

        sample_step = sample_time_ms / 1000.0
        next_deadline += sample_step
        while next_deadline < time.monotonic():
            next_deadline += sample_step

        last_cycle_started_at = cycle_started_at

    try:
        if driver_instance is not None and hasattr(driver_instance, "stop"):
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
