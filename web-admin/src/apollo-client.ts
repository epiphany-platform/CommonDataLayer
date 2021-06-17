import {
  ApolloClient,
  InMemoryCache,
  HttpLink,
} from '@apollo/client';
import { get } from 'svelte/store';
import { apiUrl } from './stores';

const cache = new InMemoryCache({
  addTypename: true,
});

const httpLink = new HttpLink({
  uri: () => get(apiUrl)
});

export default new ApolloClient({
  cache,
  link: httpLink,
  connectToDevTools: true,
});
