import { act } from "react-test-renderer";
import useIndexedDb from "../utils/database"
import { render, screen, waitFor } from "@testing-library/react";
import "fake-indexeddb/auto";
import { unmountComponentAtNode } from "react-dom";
import { useEffect, useState } from "react";


const TestSaveComponent = (props: {
    identifier: string,
    ram: Uint8Array,
}) => {
    const { save_ram } = useIndexedDb();
    const [result, setResult] = useState<string>();

    useEffect(() => {
        save_ram(props.identifier, props.ram)
            .then(() => setResult('success'))
            .catch(() => setResult('failure'));
    }, []);

    return (
        <div>{result}</div>
    );
}

const TestLoadComponent = (props: {
    identifier: string
}) => {
    const { load_ram } = useIndexedDb();
    const [ram, setRam] = useState<Uint8Array>();

    useEffect(() => {
        load_ram(props.identifier)
            .then((ram) => setRam(ram))
    })

    return (
        <div>
            {ram &&
                <div data-testid="load-div">{JSON.stringify(Array.from(ram))}</div>
            }
        </div>
    )
}


test('saves ram', async () => {
    render(<TestSaveComponent identifier="title" ram={new Uint8Array([1, 2, 3, 4, 5])}/>);

    expect(await screen.findByText(/success/i)).toBeInTheDocument();

    // check that the data really was saved
    const request = indexedDB.open('gameboy-emulator');
    let result: any;
    request.onsuccess = (e) => {
        const db: IDBDatabase = (e.target as any).result;
        const tx = db.transaction('saves');
        const getAllRequest = tx.objectStore('saves').getAll();
        getAllRequest.onsuccess = (e: any) => {
            result = e.target.result;
        }
    }

    await (waitFor(() => {
        expect(result).toEqual([
            {identifier: 'title', ram: new Uint8Array([1, 2, 3, 4, 5])}
        ]);
    }));
})


test('loads ram', async () => {
    render(<TestSaveComponent identifier="title" ram={new Uint8Array([1, 2, 3, 4, 5])}/>);
    await (waitFor(() => {}));
    render(<TestLoadComponent identifier="title"/>);
    expect(await screen.findByTestId("load-div")).toBeInTheDocument();
    expect(await screen.findByTestId("load-div")).toHaveTextContent("[1,2,3,4,5]");
})