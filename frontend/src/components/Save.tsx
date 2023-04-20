import { useCallback, useEffect, useState } from "react";

const Save = (props: {
    onSave: () => Promise<void>
}) => {
    // const [prevTime, setPrevTime] = useState<number>();
    const [recentlySaved, setRecentlySaved] = useState(false);
    const { onSave } = props;

    // reset recently saved to false after a duration
    useEffect(() => {
        let timeout: NodeJS.Timeout | undefined;
        if (recentlySaved) {
            timeout = setTimeout(() => setRecentlySaved(false), 2000);
        }
        return () => clearTimeout(timeout);
    }, [recentlySaved])

    const handleClick = useCallback(() => {
        onSave()
            .then(() => {
                // setPrevTime(0);
                setRecentlySaved(true);
                console.log('Saved');
            })
            .catch((e) => {
                console.error('Failed to save:', e);
            })
    }, [onSave])

    return (
        <div>
            <button onClick={handleClick}>Save Progress</button> {recentlySaved && <p>Saved!</p>}
        </div>
    );
}

export default Save;