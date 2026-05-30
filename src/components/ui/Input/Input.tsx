import type { InputHTMLAttributes } from "react";

type InputVariant = "default" | "search" | "tag";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  variant?: InputVariant;
}

const VARIANT_CLASS: Record<InputVariant, string> = {
  default: "",
  search: "search",
  tag: "tag-input",
};

/** Text-like input primitive. `variant` maps to the existing CSS classes. */
export default function Input({
  variant = "default",
  className,
  ...rest
}: Props) {
  const cls = [VARIANT_CLASS[variant], className].filter(Boolean).join(" ");
  return <input className={cls || undefined} {...rest} />;
}
