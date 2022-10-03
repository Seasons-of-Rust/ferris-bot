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
      driver = "docker"

      config {
        #image = "ghcr.io/seasons-of-rust/ferris-bot/ferris-bot-rust:latest"
        image = "localhost/ferris-bot:latest"
        privileged = true
      }

      resources {
        cpu    = 64
        memory = 64
      }
      
      template {
        data = <<EOH
DISCORD_TOKEN="todo_put_secret_here_or_something"
EOH
        destination = "secrets/file.env"
        env         = true
      }
    }
  }
}