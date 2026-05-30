import type { InputHTMLAttributes } from "react";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  label: string;
}

/** Checkbox with an inline label, wrapped in the existing .checkbox-row layout. */
export default function Checkbox({ label, ...rest }: Props) {
  return (
    <label className="checkbox-row">
      <input type="checkbox" {...rest} />
      {label}
    </label>
  );
}
