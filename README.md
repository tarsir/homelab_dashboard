# Issues

Small snag in the Podman port thing - from inside the Podman container, it can't see the port the host exposes for the container services. So if we have a mapping of:

8082->8080/tcp

Like in the case of `redlib`, the `docker ps --format` command inside the dashboard container only shows:

8080/tcp

And we have no way of knowing what port the service is exposed on.
