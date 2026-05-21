export type ButtonVariant =
  | "primary"
  | "secondary"
  | "success"
  | "warning"
  | "danger";

export type ButtonSize = "sm" | "md" | "lg";

export const buttonBaseClass =
  "inline-flex cursor-pointer items-center justify-center border-[0.5px] font-semibold transition duration-150 focus-visible:outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-60";

export const buttonVariantClassMap = {
  primary:
    "border-accent-600 bg-accent-600 text-white hover:border-accent-700 hover:bg-accent-700 focus-visible:ring-accent-300",
  secondary:
    "border-shell-200/80 bg-white/90 text-shell-700 hover:border-shell-300 hover:bg-shell-50 focus-visible:ring-shell-300",
  success:
    "border-emerald-600 bg-emerald-600 text-white hover:border-emerald-700 hover:bg-emerald-700 focus-visible:ring-emerald-300",
  warning:
    "border-amber-500 bg-amber-500 text-white hover:border-amber-600 hover:bg-amber-600 focus-visible:ring-amber-300",
  danger:
    "border-rose-600 bg-rose-600 text-white hover:border-rose-700 hover:bg-rose-700 focus-visible:ring-rose-300",
} as const satisfies Record<ButtonVariant, string>;

export const buttonSizeClassMap = {
  sm: {
    button:
      "h-[30px] min-w-[30px] gap-1.5 rounded-[8px] px-2.5 text-[12px] leading-none",
    iconOnlyButton:
      "h-[30px] w-[30px] gap-1.5 rounded-[8px] px-0 text-[12px] leading-none",
    icon: 14,
    iconOnlyIcon: 16,
  },
  md: {
    button:
      "h-[34px] min-w-[34px] gap-2 rounded-[8px] px-3 text-[13px] leading-none",
    iconOnlyButton:
      "h-[34px] w-[34px] gap-2 rounded-[8px] px-0 text-[13px] leading-none",
    icon: 16,
    iconOnlyIcon: 18,
  },
  lg: {
    button:
      "h-10 min-w-10 gap-2.5 rounded-[8px] px-4 text-[14px] leading-none",
    iconOnlyButton:
      "h-10 w-10 gap-2.5 rounded-[8px] px-0 text-[14px] leading-none",
    icon: 18,
    iconOnlyIcon: 20,
  },
} as const satisfies Record<
  ButtonSize,
  {
    button: string;
    iconOnlyButton: string;
    icon: number;
    iconOnlyIcon: number;
  }
>;
