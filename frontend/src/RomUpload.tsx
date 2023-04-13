import { useEffect, useState } from "react";

const RomUpload = (props: {
    onUpload: (array: Uint8Array) => void
}) => {
    const [files, setFiles] = useState<FileList | null>(null);
    const [success, setSuccess] = useState<boolean | undefined>();

    useEffect(() => {
        if (files === null) {
            return;
        }

        if (files.length === 0) {

        } else {
            const file = files[0];
            const reader = new FileReader();
            reader.onload = (evt) => {
                const result = evt.target?.result
                if (result instanceof ArrayBuffer) {
                    const array = new Uint8Array(result);
                    props.onUpload(array);
                }
            };
            reader.readAsArrayBuffer(file);
        }
    }, [files]);

    return (
        <input 
            type="file"
            onChange={e => setFiles(e.target.files)}
        />
    )
};

export default RomUpload;