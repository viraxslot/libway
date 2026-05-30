import { type FormEvent, useState } from "react";
import Button from "@/components/ui/Button/Button";
import Input from "@/components/ui/Input/Input";

interface Props {
  onAdd: (fullName: string) => Promise<void>;
}

/** Input for "owner/repo" plus an add button. */
export default function AddRepoForm({ onAdd }: Props) {
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit(ev: FormEvent) {
    ev.preventDefault();
    const trimmed = value.trim();
    if (!trimmed) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await onAdd(trimmed);
      setValue("");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <form className="add-form" onSubmit={submit}>
      <Input
        type="text"
        placeholder="owner/repo"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        disabled={busy}
        spellCheck={false}
        autoCapitalize="off"
      />
      <Button type="submit" disabled={busy || !value.trim()}>
        {busy ? "Adding…" : "Add"}
      </Button>
      {error && <p className="error">{error}</p>}
    </form>
  );
}
