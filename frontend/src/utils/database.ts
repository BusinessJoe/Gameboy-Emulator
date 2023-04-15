const dbName = "gameboy-emulator";
const savesStoreName = "saves";

const openDb = async () => {
    console.log('Opening database connection');

    const promise = new Promise<IDBDatabase>((resolve, reject) => {
        const request = indexedDB.open(dbName);
    
        request.onsuccess = (event) => {
            resolve((event.target as any).result)
        };

        request.onerror = (e) => {
            reject(e);
        };

        request.onupgradeneeded = (event) => {
            const db: IDBDatabase = (event.target as any).result;

            const objectStore = db.createObjectStore(savesStoreName, { keyPath: 'identifier' });

            objectStore.createIndex("ram", "ram", { unique: false });
        };
    });

    return promise
}

// save ram to the database
const save_ram = async (identifier: string, ram: Uint8Array) => {
    const db = await openDb();

    const promise = new Promise<void>((resolve, reject) => {
        const transaction = db.transaction([savesStoreName], 'readwrite');

        transaction.onerror = (e) => {
            reject(e);
        }

        const objectStore = transaction.objectStore(savesStoreName);

        const request = objectStore.put({ identifier, ram })

        request.onsuccess = () => {
            resolve();
        }
        request.onerror = (e) => {
            reject(e);
        }
    });

    return promise;
}

// load ram from database
const load_ram = async (identifier: string) => {
    const db = await openDb();

    const promise = new Promise<Uint8Array>((resolve, reject) => {
        const transaction = db.transaction([savesStoreName], 'readonly');
        
        const objectStore = transaction.objectStore(savesStoreName);

        const request = objectStore.get(identifier);

        request.onsuccess = () => {
            if (request.result) {
                resolve(request.result.ram);
            } else {
                reject()
            }
        }
        request.onerror = (e) => {
            reject(e);
        }
    });

    return promise;
}

export { save_ram, load_ram };