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

#[derive(PartialEq, Eq, Debug)]
pub enum DsmEvent {
    UpsBatteryMode(Host),
    UpsLowBattery(Host),
    Test(Host),
}

/// Parses an event message and returns an event if it starts with one and the rest of the unparsed message otherwise it returns an error if no event is found
pub fn parse_msg(msg: &str) -> nom::IResult<&str, DsmEvent> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case1() {
        let event_msg = "Test Message from COMPUTER1.";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::Test(Host::new("COMPUTER1".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case2() {
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
    fn case3() {
        let event_msg = "The UPS device connected to COMPUTER3 has reached low battery.";

        let (remainder_of_msg, result) = parse_msg(event_msg).expect("This one should pass");
        let expected = DsmEvent::UpsLowBattery(Host::new("COMPUTER3".to_string()));
        dbg!(remainder_of_msg); // This doesn't show by default unless the test fails
        assert_eq!(result, expected);
    }

    #[test]
    fn case4() {
        let event_msg = "Some random message about something else";

        let result: Result<_, _> = parse_msg(event_msg);
        assert!(result.is_err());
    }
}
