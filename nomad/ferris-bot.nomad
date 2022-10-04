# Sample nomad config for ferris-bot
# Caveats:
# - Requires podman driver https://github.com/hashicorp/nomad-driver-podman
#   - Podman driver requires the host has podman, and a running podman socket
#   - For the podman socket I am testing with a userspace (rootless) socket
#       systemctl --user enable --now podman.socket
#       systemctl --user status podman.socket

job "ferris-bot" {
  datacenters = ["dc1"]

  group "ferris-bot-orchestrator" {
    task "ferris-bot" {
      driver = "podman"

      config {
        image = "ghcr.io/seasons-of-rust/ferris-bot/ferris-bot-rust:latest"
        privileged = true
      }
      
      template {
        data = <<EOH
DISCORD_TOKEN="your_token_here"
EOH
        destination = "secrets/file.env"
        env         = true
      }
    }
  }
}