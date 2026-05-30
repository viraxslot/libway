import type { ButtonHTMLAttributes } from "react";

type IconButtonVariant = "remove" | "chip-remove";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant: IconButtonVariant;
}

/** Icon-only button (✕ / ×). `variant` maps to the existing CSS class. */
export default function IconButton({
  variant,
  type = "button",
  children,
  ...rest
}: Props) {
  return (
    <button type={type} className={variant} {...rest}>
      {children}
    </button>
  );
}
