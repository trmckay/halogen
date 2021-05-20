Docs are written in markdown and contain useful information and notes (mostly for myself) regarding
the project.

[Neuron](https://neuron.zettel.page/) is used to generate a static site for easy viewing, but
everything is still perfectly readable in its markdown form. A `docker-compose` file is provided
for easily running the Neuron web app.

```bash
docker compose up -d
$BROWSER http://localhost:8080
# ...
docker compose down
```
