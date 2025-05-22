import pytest

from controller_framework.core import (AppManager, MCUType)

@pytest.fixture
def app():
    return AppManager(mcu_type=MCUType.RDATA, sample_time=1000, port="COM1", baud_rate=14000)

class TestAppClass:
    @pytest.mark.parametrize(
        "setter, attr_name, entries",
        [
            ("set_actuator_vars", "actuator_vars",
            [("Act 1", "%", float), ("Act 2", "V", int), ("Act 3", "", bool)]),
            ("set_sensor_vars", "sensor_vars",
            [("Sensor 1", "ºC", float), ("Sensor 2", "V", float), ("Sensor 3", "A", float)]),
        ],
    )
    def test_set_vars(self, app, monkeypatch, setter, attr_name, entries):
        """ Ensure that setter initialize actuator_vars and sensors_vars with the correct data """

        monkeypatch.setattr(app, "random_color", lambda: "#ABC123")

        getattr(app, setter)(*entries)

        result = getattr(app, attr_name)
        expected = {
            name: {"type": typ, "value": 0, "unit": unit, "color": "#ABC123"}
            for name, unit, typ in entries
        }

        assert result == expected
