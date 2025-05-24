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

        monkeypatch.setattr(app, "_random_color", lambda: "#ABC123")

        getattr(app, setter)(*entries)

        result = getattr(app, attr_name)
        expected = {
            name: {"type": typ, "value": 0, "unit": unit, "color": "#ABC123"}
            for name, unit, typ in entries
        }

        assert result == expected

    @pytest.mark.parametrize(
        "setter, update_method, attr_name, entries, new_values, expected_values",
        [
            (
                "set_actuator_vars",
                "update_actuator_vars",
                "actuator_vars",
                [("A1", "%", float), ("A2", "V", int), ("A3", "", bool)],
                (1.23, 7, True),
                {"A1": 1.23, "A2": 7, "A3": True},
            ),
            (
                "set_sensor_vars",
                "update_sensors_vars",
                "sensor_vars",
                [("S1", "ºC", float), ("S2", "V", float), ("S3", "A", float)],
                (25.0, 12.7, 3.3),
                {"S1": 25.0, "S2": 12.7, "S3": 3.3},
            ),
        ],
    )
    def test_update_vars_success(self, app, setter, update_method, attr_name, entries, new_values, expected_values,):
        """ Ensure that the update methods update actuator_vars and sensor_vars with correct new values """

        getattr(app, setter)(*entries)
        getattr(app, update_method)(new_values) 

        result = getattr(app, attr_name)
        for name, expected in expected_values.items():
            assert result[name]["value"] == pytest.approx(expected)

    @pytest.mark.parametrize(
        "setter, update_method, entries, bad_values, expected_type_name",
        [
            ("set_actuator_vars", "update_actuator_vars", [("A", "", int)], ("oops",), "int"),
            ("set_sensor_vars",  "update_sensors_vars",  [("S", "", bool)], ("oops",), "bool"),
        ],
    )
    def test_update_vars_type_error(self, app, setter, update_method, entries, bad_values, expected_type_name):
            """Ensure that the update methods raise a TypeError when called with values of the wrong type."""

            getattr(app, setter)(*entries)

            with pytest.raises(TypeError) as exc:
                getattr(app, update_method)(*bad_values)

            assert expected_type_name in str(exc.value)

    @pytest.mark.parametrize(
        "setter, getter, updater, entries, new_values, expected_values",
        [
            (
                "set_actuator_vars",
                "get_actuator_values",
                "update_actuator_vars",
                [("A1", "%", float), ("A2", "V", int), ("A3", "", bool)],
                (1.23, 7, True),
                [1.23, 7, True]
            ),
            (
                "set_sensor_vars",
                "get_sensor_values",
                "update_sensors_vars",
                [("S1", "ºC", float), ("S2", "V", float), ("S3", "A", float)],
                (25.0, 12.7, 3.3),
                [25.0, 12.7, 3.3]
            )
        ],
    )
    def test_get_vars_values(self, app, setter, getter, updater,
                             entries, new_values, expected_values):
        """ Ensure that the getter methods return the correct values """

        getattr(app, setter)(*entries)
        getattr(app, updater)(new_values)

        result = getattr(app, getter)()

        assert result == expected_values
        