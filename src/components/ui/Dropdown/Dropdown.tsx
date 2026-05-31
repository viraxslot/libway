interface Props<T extends string> {
  value: T;
  onChange: (value: T) => void;
  options: { value: T; label: string }[];
}

export default function Dropdown<T extends string>({
  value,
  onChange,
  options,
}: Props<T>) {
  return (
    <select
      className="dropdown"
      value={value}
      onChange={(e) => onChange(e.target.value as T)}
    >
      {options.map(({ value: optionValue, label }) => (
        <option key={optionValue} value={optionValue}>
          {label}
        </option>
      ))}
    </select>
  );
}
