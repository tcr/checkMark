const { buildClientSchema, printSchema } = require("graphql");
const fs = require("fs");

const introspectionSchemaResult = JSON.parse(fs.readFileSync("./schema.json"));
const graphqlSchemaObj = buildClientSchema(introspectionSchemaResult);
const sdlString = printSchema(graphqlSchemaObj);
fs.writeFileSync('./schema.graphql', sdlString);
