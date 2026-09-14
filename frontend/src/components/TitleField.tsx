import { fieldLabel, fieldStack, inputBase } from "../ui/classes";

type Props = {
  value: string;
  disabled?: boolean;
  onChange: (value: string) => void;
};

export function TitleField({ value, disabled, onChange }: Props) {
  return (
    <div className={fieldStack}>
      <label htmlFor="title" className={fieldLabel}>
        Title <span className="font-normal text-ink-soft">(optional)</span>
      </label>
      <input
        id="title"
        type="text"
        maxLength={200}
        placeholder="Add a title that describes your video"
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        className={inputBase}
      />
      <p className="m-0 text-right text-[12px] text-ink-soft">
        {value.length}/200
      </p>
    </div>
  );
}
