# Sample nomad config for ferris-bot
# Caveats:
# - Requires podman driver https://github.com/hashicorp/nomad-driver-podman
#   - Podman driver requires the host has podman, and a running podman socket
#   - For the podman socket I am testing with a userspace (rootless) socket
#       systemctl --user enable --now podman.socket
#       systemctl --user status podman.socket
#   - Example config for nomad that I'm using with nomad-podman-driver
#```hcl
# plugin "nomad-driver-podman" {
#   config {
#     socket_path = "unix:///run/user/1000/podman/podman.sock"
#     volumes {
#       enabled      = true
#       selinuxlabel = "z"
#     }
#   }
# }
#```   
# 

job "ferris-bot" {
  datacenters = ["dc1"]

  group "ferris-bot-orchestrator" {
    task "ferris-bot" {
      driver = "podman"
      config {
        image = "ghcr.io/summer-of-rust/ferris-bot/ferris-bot-rust:latest "
        # This should be updated depending on nomad deployment
        volumes = [
          "run/user/1000/podman/podman.sock:/run/podman/podman.sock"
        ]
        user = "ferris"
      }
      
      template {
        data = <<EOH
DISCORD_TOKEN="pull_from_key_service_or_something_idk"
EOH
        destination = "secrets/file.env"
        env         = true
      }
    }
  }
}