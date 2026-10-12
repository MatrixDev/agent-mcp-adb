use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////
const PROPERTY_PREFIXES: [&str; 5] = ["usb:", "product:", "model:", "device:", "transport_id:"];

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct DeviceInfo {
    pub serial: String,
    pub name: String,
    pub product: Option<String>,
    pub model: Option<String>,
    pub state: String,
}

impl DeviceInfo {
    ////////////////////////////////////////////////////////////////////////////////
    pub fn parse_list(payload: &str) -> Vec<Self> {
        payload
            .lines()
            .skip_while(|line| !line.starts_with("List of devices"))
            .skip(1)
            .filter_map(|line| Self::from_str(line).ok())
            .collect()
    }
}

////////////////////////////////////////////////////////////////////////////////
impl FromStr for DeviceInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();
        let Some(serial) = fields.next() else {
            return Err(());
        };

        // state may contain spaces (e.g. "no permissions (...)"), it ends where properties start
        let mut state = Vec::new();
        let mut properties = Vec::new();

        for field in fields {
            if properties.is_empty() && !PROPERTY_PREFIXES.iter().any(|p| field.starts_with(p)) {
                state.push(field);
            } else {
                properties.push(field);
            }
        }

        let property = |key: &str| properties.iter().find_map(|p| p.strip_prefix(key)).map(str::to_owned);

        Ok(DeviceInfo {
            name: property("device:").unwrap_or_else(|| serial.to_owned()),
            product: property("product:"),
            model: property("model:"),
            state: state.join(" "),
            serial: serial.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::adb::device_info::DeviceInfo;

    ////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn parses_devices() {
        let output = "* daemon started successfully\n\
            List of devices attached\n\
            R5CT1234ABC            device usb:1-1 product:o1sxeea model:SM_G991B device:o1s transport_id:2\n\
            emulator-5554          unauthorized transport_id:3\n\
            0123456789             no permissions (missing udev rules); see [http://x/y] usb:1-2 transport_id:4\n\
            \n";

        let mut devices = DeviceInfo::parse_list(output).into_iter();

        let device0 = devices.next().unwrap();
        assert_eq!(device0.serial, "R5CT1234ABC");
        assert_eq!(device0.name, "o1s");
        assert_eq!(device0.product.as_deref(), Some("o1sxeea"));
        assert_eq!(device0.model.as_deref(), Some("SM_G991B"));
        assert_eq!(device0.state, "device");

        let device1 = devices.next().unwrap();
        assert_eq!(device1.serial, "emulator-5554");
        assert_eq!(device1.name, "emulator-5554");
        assert_eq!(device1.product, None);
        assert_eq!(device1.model, None);
        assert_eq!(device1.state, "unauthorized");

        let device2 = devices.next().unwrap();
        assert_eq!(device2.serial, "0123456789");
        assert_eq!(device2.name, "0123456789");
        assert_eq!(device2.product, None);
        assert_eq!(device2.model, None);
        assert_eq!(device2.state, "no permissions (missing udev rules); see [http://x/y]");

        assert!(devices.next().is_none());
    }
}
