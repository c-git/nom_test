use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
};

// See https://docs.rs/nom/latest/nom/all.html#functions for other functions that are available

#[derive(PartialEq, Eq, Debug)]
pub struct Host {
    pub name: String,
    pub battery_msg: Option<String>,
}

impl Host {
    pub fn new(name: String) -> Self {
        Self {
            name,
            battery_msg: Default::default(),
        }
    }
}

impl From<&str> for Host {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum DsmEvent {
    UpsBatteryMode(Host),
    UpsLowBattery(Host),
    UpsAcMode(Host),
    UpsConnectionLost(Host),
    UpsConnected(Host),
    Test(Host),
}

/// Parses an event message and returns an event if it starts with one and the rest of the unparsed message otherwise it returns an error if no event is found
pub fn parse_msg(msg: &str) -> nom::IResult<&str, DsmEvent> {
    alt((
        ups_battery_mode,
        ups_low_battery,
        test_msg,
        ups_ac_mode,
        ups_connection_lost,
        ups_connected,
    ))(msg)
}

fn ups_ac_mode(input: &str) -> nom::IResult<&str, DsmEvent> {
    let (input, _) = tag("The UPS device connected to ")(input)?;
    let (input, name) = is_not(" ")(input)?;
    let (input, _) = tag(" has returned to AC mode.")(input)?;
    Ok((input, DsmEvent::UpsAcMode(name.trim().into())))
}

fn ups_connection_lost(input: &str) -> nom::IResult<&str, DsmEvent> {
    let (input, name) = is_not(" ")(input)?;
    let (input, _) = tag(" has lost the connection to the UPS.")(input)?;
    Ok((input, DsmEvent::UpsConnectionLost(name.trim().into())))
}

fn ups_connected(input: &str) -> nom::IResult<&str, DsmEvent> {
    let (input, name) = is_not(" ")(input)?;
    let (input, _) = tag(" has connected to the UPS device.")(input)?;
    Ok((input, DsmEvent::UpsConnected(name.trim().into())))
}

fn ups_battery_mode(input: &str) -> nom::IResult<&str, DsmEvent> {
    let (input, _) = tag("The UPS device connected to ")(input)?;
    let (input, name) = is_not(" ")(input)?;
    let (input, _) = tag(" has entered battery mode.")(input)?;
    let (input, battery_msg) = is_not("\n")(input)?;
    Ok((
        input,
        DsmEvent::UpsBatteryMode(Host {
            name: name.trim().to_string(),
            battery_msg: Some(battery_msg.trim().to_string()),
        }),
    ))
}

fn ups_low_battery(input: &str) -> nom::IResult<&str, DsmEvent> {
    let (input, _) = tag("The UPS device connected to ")(input)?;
    let (input, name) = is_not(" ")(input)?;
    let (input, _) = tag(" has reached low battery.")(input)?;
    Ok((input, DsmEvent::UpsLowBattery(name.trim().into())))
}

fn test_msg(input: &str) -> nom::IResult<&str, DsmEvent> {
    let (input, _) = tag("Test Message from ")(input)?;
    let (input, name) = is_not(". ")(input)?;
    let (input, _) = tag(".")(input)?;
    Ok((input, DsmEvent::Test(name.trim().into())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_test() {
        let event_msg = "Test Message from COMPUTER1.";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::Test(Host::new("COMPUTER1".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case_bat_mode() {
        let event_msg =
"The UPS device connected to COMPUTER2 has entered battery mode. The battery level is 99%

From COMPUTER2";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsBatteryMode(Host {
            name: "COMPUTER2".to_string(),
            battery_msg: Some("The battery level is 99%".to_string()),
        });
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case_low_bat() {
        let event_msg = "The UPS device connected to COMPUTER3 has reached low battery.";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsLowBattery(Host::new("COMPUTER3".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case_generic_low_bat() {
        let event_msg =
"The UPS device connected to %HOSTNAME% has reached low battery. Charge your UPS or connect it to a power outlet.

All services are shutting down in the meantime and will be restarted once the UPS is recovered.

From %HOSTNAME%";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsLowBattery(Host::new("%HOSTNAME%".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case_is_err() {
        let event_msg = "Some random message about something else";

        let result: Result<_, _> = parse_msg(event_msg);
        assert!(result.is_err());
    }

    #[test]
    fn case_generic_bat_mode() {
        let event_msg =
            "The UPS device connected to %HOSTNAME% has entered battery mode. %BATTERY_STRING%

From %HOSTNAME%";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsBatteryMode(Host {
            name: "%HOSTNAME%".to_string(),
            battery_msg: Some("%BATTERY_STRING%".to_string()),
        });
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case_generic_ac_mode() {
        let event_msg = "The UPS device connected to %HOSTNAME% has returned to AC mode.

From %HOSTNAME%";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsAcMode(Host::new("%HOSTNAME%".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case_generic_connection_lost() {
        let event_msg =
"%HOSTNAME% has lost the connection to the UPS. Please go to DSM > Control Panel > Hardware & Power > UPS to check the server settings.

From %HOSTNAME%";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsConnectionLost(Host::new("%HOSTNAME%".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case_generic_connected() {
        let event_msg = "%HOSTNAME% has connected to the UPS device.

From %HOSTNAME%";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsConnected(Host::new("%HOSTNAME%".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }
}
