# AppPark

📱 AppPark is a simple iOS app metadata extractor and server 📱

## Features

- Automatic app name, version, build datetime, icon, size and bundle identifier extraction from an IPA file
- Web UI app installer
- Manifest.plist and install link generation
- File System watcher and auto-reload

## Configuration

### Options

```
USAGE:
    app-park [FLAGS] [OPTIONS]

FLAGS:
    -h, --help             Prints help information
    -V, --version          Prints version information
    -w, --watch-storage    

OPTIONS:
    -a, --address <address>     [default: 127.0.0.1]
    -p, --port <port>           [default: 8080]
    -s, --storage <storage>     [default: .]
```

### Reverse-proxy

If you host AppPark behind a reverse-proxy, make sure to forward original host by setting the `X-Forwarded-Host` header.

### Docker

If you prefer to run AppPark as a Docker container, you can either build the image yourself using the Dockerfile available in this repo, or you can use the [image](https://github.com/scotow/app-park/pkgs/container/app-park%2Fapp-park) built by the GitHub action.

```
docker run -p 8080:8080 -v $(pwd):/apps ghcr.io/scotow/app-park/app-park:latest -s /apps
```

Please read [Binding to all interfaces](#binding-to-all-interfaces) if you can't reach the process from outside the image.

### Binding to all interfaces

By default, AppPark will only listen on the loopback interface, aka. 127.0.0.1. If you **don't** want to host AppPark behind a reverse proxy or if you are using the Docker image, you should specify the `0.0.0.0` address by using the `-a | --address` option.