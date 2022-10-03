use crate::model::configurable::*;

/// Sets the container image to pull
pub const CONTAINER_IMAGE: &ConfigurableItem<&str> = &ConfigurableItem {
    environment_variable: "CONTAINER_IMAGE",
    default_value: "ghcr.io/theconner/rustbot-runner:latest",
};

/// Sets the maximum amount of virtual CPUs available to the child container
pub const CONTAINER_CPU: &ConfigurableItem<&str> = &ConfigurableItem {
    environment_variable: "CONTAINER_CPU",
    default_value: "0.5", // you get 1/2 of a cpu, i'm being generous
};

/// Sets the maximum amount of memory available to the child container
pub const CONTAINER_MEMORY: &ConfigurableItem<&str> = &ConfigurableItem {
    environment_variable: "CONTAINER_MEMORY",
    default_value: "100m",
};

// TODO: add settings for CPU scheduler although, i'm unsure if podman supports
// userspace containers with different schedulers

/// Sets the maximum amount of swap usable to the child container
pub const CONTAINER_SWAP: &ConfigurableItem<&str> = &ConfigurableItem {
    environment_variable: "CONTAINER_SWAP",
    default_value: "5m",
};

/// How long can a container run for?
pub const CONTAINER_MAX_RUNTIME: &ConfigurableItem<u64> = &ConfigurableItem {
    environment_variable: "CONTAINER_MAX_RUNTIME",
    default_value: 5000,
};

/// Tells the bot if it's running in a container this will influence flags it
/// chooses for child containers available values: false,true
pub const IS_RUNNING_IN_CONTAINER: &ConfigurableItem<bool> = &ConfigurableItem {
    environment_variable: "IS_RUNNING_IN_CONTAINER",
    default_value: false,
};

// Specifies whether the container should have networking
pub const CONTAINER_NETWORK: &ConfigurableItem<&str> = &ConfigurableItem {
    environment_variable: "CONTAINER_NETWORK",
    default_value: "none",
};

// Maximum amount of PIDs available to a container
pub const CONTAINER_PIDS: &ConfigurableItem<u64> = &ConfigurableItem {
    environment_variable: "MAX_PIDS",
    default_value: 64,
};
