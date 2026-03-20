from __future__ import annotations

import importlib.util
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path
from types import ModuleType
from typing import Any


def load_runner_module() -> ModuleType:
    runner_path = Path(__file__).with_name("runner.py")
    spec = importlib.util.spec_from_file_location("senamby_runtime_runner", runner_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("Falha ao carregar runner.py para testes")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


runner = load_runner_module()


class RunnerContractTests(unittest.TestCase):
    def build_bootstrap(self, root: Path) -> Any:
        plant_variables = [
            runner.VariableSpec(
                id="sensor_1",
                name="Sensor 1",
                type="sensor",
                unit="C",
                setpoint=42.0,
                pv_min=0.0,
                pv_max=100.0,
                linked_sensor_ids=[],
            ),
            runner.VariableSpec(
                id="actuator_1",
                name="Actuator 1",
                type="actuator",
                unit="%",
                setpoint=0.0,
                pv_min=0.0,
                pv_max=100.0,
                linked_sensor_ids=["sensor_1"],
            ),
        ]

        variables_by_id = {variable.id: variable for variable in plant_variables}
        plant = runner.PlantContext(
            id="plant_1",
            name="Plant 1",
            variables=plant_variables,
            variables_by_id=variables_by_id,
            sensors=runner.IOGroup(
                ids=["sensor_1"],
                count=1,
                variables=[variables_by_id["sensor_1"]],
                variables_by_id={"sensor_1": variables_by_id["sensor_1"]},
            ),
            actuators=runner.IOGroup(
                ids=["actuator_1"],
                count=1,
                variables=[variables_by_id["actuator_1"]],
                variables_by_id={"actuator_1": variables_by_id["actuator_1"]},
            ),
            setpoints={"sensor_1": 42.0, "actuator_1": 0.0},
        )

        runtime = runner.RuntimeContext(
            id="rt_1",
            timing=runner.RuntimeTiming(
                owner="runtime",
                clock="monotonic",
                strategy="deadline",
                sample_time_ms=100,
            ),
            supervision=runner.RuntimeSupervision(
                owner="rust",
                startup_timeout_ms=12000,
                shutdown_timeout_ms=4000,
            ),
            paths=runner.RuntimePaths(
                runtime_dir=str(root / "runtime"),
                venv_python_path=str(root / ".venv" / "bin" / "python"),
                runner_path=str(root / "runtime" / "runner.py"),
                bootstrap_path=str(root / "runtime" / "bootstrap.json"),
            ),
        )

        driver_dir = root / "driver_plugin"
        driver_dir.mkdir()
        (driver_dir / "main.py").write_text(
            textwrap.dedent(
                """
                from typing import Any, Dict

                class ContractDriver:
                    def __init__(self, context: Any) -> None:
                        if hasattr(context, "runtime"):
                            raise RuntimeError("driver context leaked runtime")
                        self.context = context
                        self.context_keys = set(vars(context).keys())

                    def connect(self) -> bool:
                        return True

                    def stop(self) -> bool:
                        return True

                    def read(self) -> Dict[str, Dict[str, float]]:
                        return {
                            "sensors": {"sensor_1": 1.0},
                            "actuators": {"actuator_1": 0.0},
                        }

                    def write(self, outputs: Dict[str, float]) -> bool:
                        return True
                """
            ).strip()
            + "\n",
            encoding="utf-8",
        )

        controller_dir = root / "controller_plugin"
        controller_dir.mkdir()
        (controller_dir / "main.py").write_text(
            textwrap.dedent(
                """
                from typing import Any, Dict

                class ContractController:
                    def __init__(self, context: Any) -> None:
                        if hasattr(context, "runtime"):
                            raise RuntimeError("controller context leaked runtime")
                        self.context = context
                        self.context_keys = set(vars(context).keys())
                        self.controller_keys = set(vars(context.controller).keys())

                    def connect(self) -> bool:
                        return True

                    def stop(self) -> bool:
                        return True

                    def compute(self, snapshot: Dict[str, Any]) -> Dict[str, float]:
                        return {self.context.controller.output_variable_ids[0]: 0.0}
                """
            ).strip()
            + "\n",
            encoding="utf-8",
        )

        return runner.RuntimeBootstrap(
            driver=runner.DriverMetadata(
                plugin_id="driver_plugin",
                plugin_name="Driver Plugin",
                plugin_dir=str(driver_dir),
                source_file="main.py",
                class_name="ContractDriver",
                config={"port": "COM1"},
            ),
            controllers=[
                runner.ControllerMetadata(
                    id="ctrl_1",
                    plugin_id="controller_plugin",
                    plugin_name="Controller Plugin",
                    plugin_dir=str(controller_dir),
                    source_file="main.py",
                    class_name="ContractController",
                    name="Controller 1",
                    controller_type="PID",
                    active=True,
                    input_variable_ids=["sensor_1"],
                    output_variable_ids=["actuator_1"],
                    params={
                        "kp": runner.ControllerParamSpec(
                            key="kp",
                            type="number",
                            value=1.2,
                            label="Kp",
                        )
                    },
                )
            ],
            plant=plant,
            runtime=runtime,
        )

    def test_driver_context_exposes_only_config_and_plant(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            bootstrap = self.build_bootstrap(Path(tmp_dir))
            context = runner.build_driver_plugin_context(bootstrap)

        self.assertEqual(set(vars(context).keys()), {"config", "plant"})
        self.assertFalse(hasattr(context, "runtime"))

    def test_controller_context_uses_minimum_public_shape(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            bootstrap = self.build_bootstrap(Path(tmp_dir))
            context = runner.build_controller_plugin_context(
                bootstrap.controllers[0],
                bootstrap.plant,
            )

        self.assertEqual(set(vars(context).keys()), {"controller", "plant"})
        self.assertFalse(hasattr(context, "runtime"))
        self.assertEqual(
            set(vars(context.controller).keys()),
            {
                "id",
                "name",
                "controller_type",
                "input_variable_ids",
                "output_variable_ids",
                "params",
            },
        )

    def test_snapshot_controller_omits_internal_loader_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            bootstrap = self.build_bootstrap(Path(tmp_dir))
            snapshot = runner.build_controller_snapshot(
                cycle_id=1,
                cycle_started_at=123.456,
                dt_ms=100.0,
                plant=bootstrap.plant,
                controller=bootstrap.controllers[0],
                sensors={"sensor_1": 40.0},
                actuators={"actuator_1": 10.0},
            )

        self.assertEqual(
            set(snapshot["controller"].keys()),
            {
                "id",
                "name",
                "controller_type",
                "input_variable_ids",
                "output_variable_ids",
                "params",
            },
        )
        self.assertNotIn("plugin_id", snapshot["controller"])
        self.assertNotIn("plugin_name", snapshot["controller"])
        self.assertNotIn("active", snapshot["controller"])

    def test_engine_loads_plugins_with_internal_bootstrap_and_public_context(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            bootstrap = self.build_bootstrap(Path(tmp_dir))
            engine = runner.PlantRuntimeEngine(bootstrap)
            try:
                engine.start()

                driver_instance = engine.driver_instance
                self.assertIsNotNone(driver_instance)
                self.assertEqual(driver_instance.context_keys, {"config", "plant"})
                self.assertFalse(hasattr(driver_instance.context, "runtime"))

                self.assertEqual(len(engine.controllers), 1)
                controller_instance = engine.controllers[0].instance
                self.assertEqual(controller_instance.context_keys, {"controller", "plant"})
                self.assertFalse(hasattr(controller_instance.context, "runtime"))
                self.assertEqual(
                    controller_instance.controller_keys,
                    {
                        "id",
                        "name",
                        "controller_type",
                        "input_variable_ids",
                        "output_variable_ids",
                        "params",
                    },
                )
            finally:
                engine.stop()


if __name__ == "__main__":
    unittest.main()
