import { RemoteData, loaded } from "./models";

export function mapLoaded<T, U>(
  data: RemoteData<T>,
  mapper: (t: T) => U
): RemoteData<U> {
  if (data.status === "loaded") {
    return loaded(mapper(data.data));
  } else {
    return data;
  }
}

export function getLoaded<T>(data: RemoteData<T>): T | null {
  return data.status === "loaded" ? data.data : null;
}

export function validUuid(id: string): boolean {
  const uuidRegex =
    /^[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}$/i;
  return uuidRegex.test(id);
}

// export function debounce(callback: (...args: any[]) => void, interval: number): (...args: any[]) => void {
//   let debounceTimeoutId: number;

//   return function(...args) {
//     clearTimeout(debounceTimeoutId);
//     debounceTimeoutId = setTimeout(() => callback.apply(this, args), interval);
//   };
// }
