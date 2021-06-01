# OsmTasksFrontend

This project is built with [Angular CLI](https://github.com/angular/angular-cli) version 8.3.24.

## Prerequisites for Development

An active LTS or maintenance LTS version of [Node.js](https://nodejs.org/en/about/releases/) 

npm package manager, you can use this [guide](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm/)

## Development server

Run `ng serve` to start the dev server. Navigate to `http://localhost:4200/`.

## Generate API Stubs

We used [OpenAPI 3](https://swagger.io/specification/) to specify the API interfaces between the backend and the frontend. By using [ng-openapi-gen](https://www.npmjs.com/package/ng-openapi-gen) we can generate model interfaces and web service clients from the backend OpenAPI specification.

With the backend running on  ` http://localhost:8000` run  ` ng run gen` to generate new model interfaces and web service clients. Files are generated to the  `generated/ ` folder
