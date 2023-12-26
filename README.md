# Kukuana Wash CLI Documentation

## Overview
The `Kukuana Wash CLI` is a streamlined wrapper for the wasmCloud `wash` CLI, designed to enhance the developer experience during local development of wasmCloud applications. It avoids Docker complexities by working directly with a local `wash-cli`.

## Usage
```zsh
kuwash dev [path_to_wadm].yaml
```

or run the install script.

```zsh
./install.sh #
```

It will build and copy the binary to /usr/local/bin/kukuana-wash.
Then you run run the following:
```zsh
kuwash --help
```

## Purpose
The wrapper addresses specific issues with the standard `wash cli`'s `dev` mode, such as creating a new host for each session and failing to clean up after receiving a `SIGINT` signal.

## Functionality
The `Kukuana Wash CLI` simplifies several key operations:

1. **Building Actors & Providers**:
   - `wash build` for actors.
   - `make` for providers.

2. **Application Management**:
   - `wash app put`, `deploy`, `undeploy`, and `delete` for application lifecycle management.

3. **Stopping Components**:
   - `wash stop` to halt actors and providers.

4. **Inspection**:
   - `wash inspect` for examining components.

5. **Simplified Execution**:
   - Utilizes a `--simple` flag with a `.wadm` file to perform the above operations through the underlying wasmCloud wash CLI.

## Future Direction
- Integration with `wash-lib` for direct interaction with `NATS` and `wasm-time`.

## Working Mechanism
1. **Manifest Reading**: Parses the `.wadm` file into a local `Manifest` variable within the `DevCommand` struct.

2. **State Setup**: Identifies file-based images in the manifest, inspects them for details (claims, IDs), and stores them in a `HashMap<String, (Component, ComponentClaims)>`.

3. **Local Image Check**: Exits if no local file images are found (as `dev mode` should not run with remote images).

4. **Component Building**: Constructs all found Components (Actors, Capability Providers).

5. **Deployment**: Deploys the application manifest.

6. **File Watching & Live Updating**:
   - Monitors file changes.
   - On changes: rebuilds the component and stops it, triggering a self-healing restart with the latest image.

## Recommendations for Future Development
- **Docker Support**: Consider adding Docker compatibility for broader use cases.
- **Enhanced Documentation**: Create comprehensive guides for ease of use.
- **Performance Tuning**: Optimize file watching and component management for resource efficiency.
- **Community Engagement**: Solicit feedback from the wasmCloud community for continual improvement.
- **Diverse Environment Testing**: Test the tool in varied development settings.
