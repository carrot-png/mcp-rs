# mcp-rs

An MCP (model context protocol) server for various small tools.

## Tools

- **Python Sandbox**: Run Python code using the Monty interpreter.
- **Date Time**: Get the local date and time.
- **WebSearch**: Connect to a [SearXNG](https://github.com/searxng/searxng) instance for web search capabilities.

## Configuration

Ensure the following environment variables are set:
```bash
# .env
BASE_URL="your.host" # If you are using a reverse proxy
MCP_PORT="3000" # Host port for docker
SEARXNG_URL="http://localhost:8080" # Search endpoint
```

### SearXNG
If you are [hosting SearXNG locally](https://docs.searxng.org/admin/installation-docker.html#installation-container) ensure the json search format is enabled:
```yaml
# settings.yml
search:
  formats:
    - html
    - json # Ensure json is enabled here.
```

## Running

### Locally

```bash
cargo run --release
```

The server will be available at `http://localhost:3000/mcp`.

### Using Docker

```bash
docker-compose up -d
```

