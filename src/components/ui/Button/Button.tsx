import type { ButtonHTMLAttributes } from "react";

type ButtonVariant = "primary" | "secondary" | "danger" | "link";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

const VARIANT_CLASS: Record<ButtonVariant, string> = {
  primary: "",
  secondary: "secondary",
  danger: "danger",
  link: "link",
};

/** Button primitive. `variant` maps to the existing CSS classes; primary has none. */
export default function Button({
  variant = "primary",
  className,
  type = "button",
  children,
  ...rest
}: Props) {
  const cls = [VARIANT_CLASS[variant], className].filter(Boolean).join(" ");
  return (
    <button type={type} className={cls || undefined} {...rest}>
      {children}
    </button>
  );
}
