#![no_std]
#![no_main]

use defmt_semihosting as _;
use panic_probe as _;

// Custom defmt panic handler for tests
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {

    use sensor_swarm::commands::parser::*;

    #[test]
    fn test_parse_sensors_command() {
        let parser = CommandParser::new();

        // Test "sensors" command
        let result = parser.parse("sensors");
        defmt::assert!(result == Command::ReadSensors);

        // Test "read_sensors" command
        let result = parser.parse("read_sensors");
        defmt::assert!(result == Command::ReadSensors);
    }

    #[test]
    fn test_parse_temperature_commands() {
        let parser = CommandParser::new();

        // Test "temp" command
        let result = parser.parse("temp");
        defmt::assert!(result == Command::ReadSensorType(SensorType::Temperature));

        // Test "temperature" command
        let result = parser.parse("temperature");
        defmt::assert!(result == Command::ReadSensorType(SensorType::Temperature));
    }

    #[test]
    fn test_parse_sensor_type_commands() {
        let parser = CommandParser::new();

        // Test humidity command
        let result = parser.parse("humidity");
        defmt::assert!(result == Command::ReadSensorType(SensorType::Humidity));

        // Test light command
        let result = parser.parse("light");
        defmt::assert!(result == Command::ReadSensorType(SensorType::Light));

        // Test pressure command
        let result = parser.parse("pressure");
        defmt::assert!(result == Command::ReadSensorType(SensorType::Pressure));
    }

    #[test]
    fn test_parse_system_commands() {
        let parser = CommandParser::new();

        // Test debug commands
        let result = parser.parse("debug");
        defmt::assert!(result == Command::GetDebugInfo);

        let result = parser.parse("debug_info");
        defmt::assert!(result == Command::GetDebugInfo);

        // Test status command
        let result = parser.parse("status");
        defmt::assert!(result == Command::GetStatus);

        // Test ping command
        let result = parser.parse("ping");
        defmt::assert!(result == Command::Ping);
    }

    #[test]
    fn test_parse_help_commands() {
        let parser = CommandParser::new();

        // Test help command
        let result = parser.parse("help");
        defmt::assert!(result == Command::Help);

        // Test "?" command
        let result = parser.parse("?");
        defmt::assert!(result == Command::Help);
    }

    #[test]
    fn test_parse_version_command() {
        let parser = CommandParser::new();

        let result = parser.parse("version");
        defmt::assert!(result == Command::Version);
    }

    #[test]
    fn test_parse_reboot_commands() {
        let parser = CommandParser::new();

        // Test reboot command
        let result = parser.parse("reboot");
        defmt::assert!(result == Command::Reboot);

        // Test DFU commands
        let result = parser.parse("dfu");
        defmt::assert!(result == Command::RebootToDfu);

        let result = parser.parse("reboot_dfu");
        defmt::assert!(result == Command::RebootToDfu);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let parser = CommandParser::new();

        // Test uppercase
        let result = parser.parse("SENSORS");
        defmt::assert!(result == Command::ReadSensors);

        // Test mixed case
        let result = parser.parse("TeMpErAtUrE");
        defmt::assert!(result == Command::ReadSensorType(SensorType::Temperature));

        // Test lowercase
        let result = parser.parse("ping");
        defmt::assert!(result == Command::Ping);
    }

    #[test]
    fn test_parse_unknown_command() {
        let parser = CommandParser::new();

        let result = parser.parse("unknown_command");
        match result {
            Command::Unknown(cmd) => {
                defmt::assert!(cmd.as_str() == "unknown_command");
            }
            _ => {
                defmt::panic!("Expected Unknown command");
            }
        }
    }

    #[test]
    fn test_parse_empty_command() {
        let parser = CommandParser::new();

        let result = parser.parse("");
        match result {
            Command::Unknown(cmd) => {
                defmt::assert!(cmd.as_str() == "");
            }
            _ => {
                defmt::panic!("Expected Unknown command for empty string");
            }
        }
    }

    #[test]
    fn test_parse_partial_matches() {
        let parser = CommandParser::new();

        // Test that partial matches don't work (should be Unknown)
        let result = parser.parse("sen"); // partial of "sensors"
        match result {
            Command::Unknown(_) => {
                // This is expected
            }
            _ => {
                defmt::panic!("Expected Unknown command for partial match");
            }
        }

        let result = parser.parse("tem"); // partial of "temp"
        match result {
            Command::Unknown(_) => {
                // This is expected
            }
            _ => {
                defmt::panic!("Expected Unknown command for partial match");
            }
        }
    }

    #[test]
    fn test_command_parser_default() {
        let parser = CommandParser;

        let result = parser.parse("ping");
        defmt::assert!(result == Command::Ping);
    }
}
