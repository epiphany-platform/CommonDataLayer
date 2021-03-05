import { get, writable } from "svelte/store";
import { notLoaded, RemoteData, Schema } from "./models";

export const schemas = writable<RemoteData<Schema[]>>(notLoaded);

export const apiUrl = writable(localStorage.getItem("api-url") || "");

apiUrl.subscribe(url => {
  localStorage.setItem("api-url", url);
});

if (get(apiUrl) === "") {
  apiUrl.set("http://localhost:50106");
}

export const darkMode = writable(
  localStorage.getItem("dark-mode") === "true",
);

darkMode.subscribe(isDarkMode => {
  localStorage.setItem("dark-mode", JSON.stringify(isDarkMode));
});
