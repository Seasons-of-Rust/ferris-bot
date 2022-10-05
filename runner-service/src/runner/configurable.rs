/// Struct for holding a Configuration item of type T
/// This is a generic struct that can be used to hold any type of configuration item that can be pulled out of environment variables
pub struct ConfigurableItem<T> {
    pub environment_variable: &'static str,
    pub default_value: T,
}

/// The value of the configuration item of type T
pub trait ConfigurableValue<T> {
    fn value(&self) -> T;
}

impl ConfigurableValue<String> for ConfigurableItem<&'static str> {
    fn value(&self) -> String {
        std::env::var(self.environment_variable).unwrap_or_else(|_| self.default_value.to_string())
    }
}

impl ConfigurableValue<u64> for ConfigurableItem<u64> {
    fn value(&self) -> u64 {
        // TODO: there is probably a nicer way of doing this
        std::env::var(self.environment_variable)
            .unwrap_or_else(|_| self.default_value.to_string())
            .parse()
            .unwrap_or(self.default_value)
    }
}

impl ConfigurableValue<bool> for ConfigurableItem<bool> {
    fn value(&self) -> bool {
        std::env::var(self.environment_variable)
            .unwrap_or_else(|_| self.default_value.to_string())
            .parse()
            .unwrap_or(self.default_value)
    }
}
