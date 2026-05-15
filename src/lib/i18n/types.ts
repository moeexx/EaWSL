export const SUPPORTED_LANGUAGES = ["en-US", "zh-CN"] as const;

export type AppLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export type DeepPartial<T> = T extends (...args: any[]) => unknown
  ? T
  : T extends object
    ? { [K in keyof T]?: DeepPartial<T[K]> }
    : T;

export type WidenCopy<T> = T extends (...args: any[]) => unknown
  ? T
  : T extends string
    ? string
    : T extends number
      ? number
      : T extends boolean
        ? boolean
        : T extends object
          ? { readonly [K in keyof T]: WidenCopy<T[K]> }
          : T;
