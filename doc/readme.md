# Pentoxide documentation

## Build with local Sphinx environment

```bash
$ make <pdf|html|etc.>
```

## Build with Docker

```bash
$ docker build -t sphinx .
$ docker run -v $(pwd):/sphinx sphinx make <pdf|html|etc.>
```
