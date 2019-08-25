import {
  Environment,
  Network,
  RecordSource,
  Store,
  Observable,
  GraphQLResponse
} from 'relay-runtime';

// Define a function that fetches the results of an operation (query/mutation/etc)
// and returns its results as a Promise:
function fetchQuery(
  operation,
  variables,
  cacheConfig,
  uploadables,
) {
  const liveWhitelist = new Set([
    'AppQuery',
    'AppPollerQuery',
  ]);

  return Observable.create(source => {
    async function graphqlFetch() {
      const response = await fetch('/graphql', {
        method: 'POST',
        headers: {
          // Add authentication and other headers here
          'content-type': 'application/json'
        },
        body: JSON.stringify({
          query: operation.text, // GraphQL text from input
          variables,
        }),
      });
      const json = await response.json();
      source.next(json);
    }

    graphqlFetch();
    if (liveWhitelist.has(operation.name)) {
      console.info('polling at interval', 3000);
      setInterval(graphqlFetch, 3000);
    }
    return () => {};
  });
}

const source = new RecordSource();
const store = new Store(source);
const network = Network.create(fetchQuery);
const handlerProvider = null;

export default new Environment({
  handlerProvider, // Can omit.
  network,
  store,
});
