# Ferris-Bot

## Development

You will need a discord bot token with the appropriate scopes and permissions to use slash commands. Create a `.env` file with the following content:
```
DISCORD_TOKEN=YOUR_TOKEN_HERE
```

Ferris-Bot supports running in 2 different modes
- **Run directly on host**: the bot will run directly on your host as a regular process. Your host needs [podman](https://podman.io/) installed as the bot invokes podman to run containers for executing code. To run Ferris-Bot this way, simply use `cargo run`
- **Run as a container**: the bot will run as a container and spawn nested containers for executing code. Running this way is a little more work:

  First, to build the image locally (this will compile the bot in release mode, may take a long time):
  ```bash
  podman build -t ferrisbot:latest .
  ```

  After the container is built, it can be ran using rootless Podman. To do so, you need to ensure a userspace [podman socket](https://docs.podman.io/en/latest/markdown/podman-system-service.1.html) is running.
  ```bash
  # Start a rootless podman socket (one-time, won't persist after reboots)
  $ systemctl --user start podman.socket
  # Start a rootless podman socket (persistent, will restart after reboots)
  $ systemctl --user enable --now podman.socket
  # Get the status of the socket
  $ systemctl --user status podman.socket
  ```

  Then, to run the locally tagged container with the socket passed to it (the uid being used for the mapping is the user invoking podman, `systemctl --user status podman.socket`):
  ```bash
   podman run \
      --rm \
      --user ferris \
      -v /run/user/1000/podman/podman.sock:/run/podman/podman.sock \
      --security-opt label=disable \
      --env-file .env \
      localhost/ferris-bot:latest
  ```

  There are security considerations to actually deploy this: exposing the podman socket is insecure as hypothetically if the ferrisbot container gets taken over, an adversary could spawn arbitrary containers on the host. We partially mitigate major threats by targeting *rootless* Podman, thus (hopefully) preventing privelege escallation as we have seen with exposing a *rootful* podman socket. 

  Therefore, it is recommended that a deployment of this is done on a isolated VM, or at the very least, an isolated account running its own seperate podman socket that is not used by anything else.
