import { type ChangeEvent, type Ref, useCallback, useImperativeHandle, useRef } from "react";

export function FilePicker({ ref, acceptedFileTypes, allowsMultiple = false, onSelect }: FilePickerProps) {
  const input = useRef<HTMLInputElement>(null);

  useImperativeHandle(
    ref,
    () => ({
      open() {
        const element = input.current;

        if (element !== null) {
          element.value = "";
          element.click();
        }
      },
    }),
    [],
  );

  const select = useCallback((event: ChangeEvent<HTMLInputElement>) => onSelect(event.currentTarget.files), [onSelect]);

  return (
    <input
      ref={input}
      type="file"
      hidden
      accept={acceptedFileTypes.join(",")}
      multiple={allowsMultiple}
      onChange={select}
    />
  );
}

export interface FilePickerHandle {
  open(): void;
}

export interface FilePickerProps {
  ref: Ref<FilePickerHandle>;
  acceptedFileTypes: readonly string[];
  allowsMultiple?: boolean;
  onSelect: (files: FileList | null) => void;
}
