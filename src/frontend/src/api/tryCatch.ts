import axios from "axios";

export async function tryCatch<T>(promise: Promise<T>): Promise<[T, null] | [null, string]> {
  try {
    const data = await promise;
    return [data, null];
  } catch (err) {
    if (err instanceof axios.AxiosError) {
      return [null, (err.response?.data as { error: string })?.error ?? "Something went wrong"];
    }
    return [null, "Something went wrong"];
  }
}
