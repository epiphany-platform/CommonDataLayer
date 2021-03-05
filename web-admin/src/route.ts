import { writable, derived } from "svelte/store";

export interface SchemasRoute {
  page: "schemas";
  id?: string;
  query?: string;
  version?: string;
  creating?: boolean;
}

export type QueryType = "single" | "multiple" | "schema";

export interface QueryRoute {
  page: "query";
  by?: QueryType;
}

export type Route =
  | { page: "home" }
  | { page: "insert" }
  | { page: "settings" }
  | QueryRoute
  | SchemasRoute;

function parseCurrentRoute(): Route | null {
  const hash = location.hash;
  const params = new URLSearchParams(location.hash.split("?")[1] || "");

  if (hash === "" || hash === "#/") {
    return { page: "home" };
  } else if (hash.startsWith("#/insert")) {
    return { page: "insert" };
  } else if (hash.startsWith("#/query")) {
    const by = params.get("by") || undefined;
    if (["single", "multiple", "schema"].includes(by)) {
      return { page: "query", by: (by as QueryType) };
    } else {
      return { page: "query" };
    }
  } else if (hash.startsWith("#/settings")) {
    return { page: "settings" };
  } else if (hash.startsWith("#/schemas")) {
    const id = params.get("id") || undefined;
    const query = params.get("query") || undefined;
    const version = params.get("version") || undefined;
    const creating = params.get("creating") === "true";

    return { page: "schemas", id, version, query, creating };
  }

  return null;
}


export function routeToString(route: Route): string {
  if (route.page === "home") {
    return "#/";
  } else if (route.page === "insert") {
    return "#/insert";
  } else if (route.page === "settings") {
    return "#/settings";
  } else if (route.page === "query") {
    if (route.by) {
      return `#/query?by=${route.by}`;
    } else {
      return "#/query";
    }
  } else {
    const params = [
      ["id", route.id],
      ["version", route.version],
      ["query", route.query],
      ["creating", route.creating ? "true" : undefined],
    ].filter(([_, value]) => value).map(([key, value]) => `${key}=${value}`);

    if (params.length) {
      return `#/schemas?${params.join("&")}`;
    } else {
      return "#/schemas";
    }
  }
}

const innerRoute = writable(parseCurrentRoute());

window.onpopstate = () => {
  innerRoute.set(parseCurrentRoute());
};

export function pushRoute(to: Route) {
  history.pushState(null, "", routeToString(to));
  innerRoute.set(to);
}

export function replaceRoute(to: Route) {
  history.replaceState(null, "", routeToString(to));
  innerRoute.set(to);
}

export const route = derived(innerRoute, $inner => $inner);
