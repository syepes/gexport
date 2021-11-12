## Grafana Exporter (Datasources and Dashboards)

[![rust](https://github.com/syepes/gexport/workflows/rust/badge.svg)](https://github.com/syepes/gexport/actions?query=branch%3Amain+event%3Apush+workflow%3Arust)
[![release](https://github.com/syepes/gexport/workflows/release/badge.svg)](https://github.com/syepes/gexport/actions?query=branch%3Amain+event%3Apush+workflow%3Arelease)

## Functionality

This tool will allow you to export in JSON (pretty print) all the Grafana Datasources and Dashboards from all the Organizations.

Once you have these files on your filesystem you can put them in your favorite version control solution.

## Usage (Docker)

    # List available options
    docker run -it --rm syepes/gexport:latest --help

    # Parameters can be either set by arguments or environment variables
    docker run -d --name ge -v $pwd/export:/app/export syepes/gexport:latest -h http://host:port -u usr -p pwd
    docker run -d --name ge -v $pwd/export:/app/export -e HOST=http://host:port -e AUTH_USR=usr -e AUTH_PWD=pwd syepes/gexport:latest
