// Create an instance of a db object for us to store the open database in
let db;

// Open our database; it is created if it doesn't already exist
// (see the upgradeneeded handler below)
const openRequest = window.indexedDB.open("saves_db", 1);

// error handler signifies that the database didn't open successfully
openRequest.addEventListener("error", () =>
  console.error("Database failed to open")
);

// success handler signifies that the database opened successfully
openRequest.addEventListener("success", () => {
  console.log("Database opened successfully");

  // Store the opened database object in the db variable. This is used a lot below
  db = openRequest.result;
});

// Set up the database tables if this has not already been done
openRequest.addEventListener("upgradeneeded", (e) => {
    // Grab a reference to the opened database
    db = e.target.result;
  
    // Create an objectStore in our database to store notes and an auto-incrementing key
    // An objectStore is similar to a 'table' in a relational database
    const objectStore = db.createObjectStore("saves_os", { keyPath: "title" });
  
    // Define what data items the objectStore will contain
    //objectStore.createIndex("title", "title", { unique: true });
    objectStore.createIndex("ram", "ram", { unique: false });
  
    console.log("Database setup complete");
});
  


function save(gameboy) {
    const name = gameboy.game_name();
    const ram = gameboy.get_save();
    console.log("ram size: %d KB", ram.length / 1024);
    const newItem = { "title": name, "ram": ram };

    // open a read/write db transaction, ready for adding the data
    const transaction = db.transaction(["saves_os"], "readwrite");

    // call an object store that's already been added to the database
    const objectStore = transaction.objectStore("saves_os");

    // Make a request to add our newItem object to the object store, or update it if it already exists
    const addRequest = objectStore.put(newItem);

    // Report on the success of the transaction completing, when everything is done
    transaction.addEventListener("complete", () => {
        console.log("Transaction completed: database modification finished.");
    });

    transaction.addEventListener("error", (e) => {
        console.log(e)
        console.log("Transaction not opened due to error")
    });
}

function load(gameboy) {
    const name = gameboy.game_name();

    const transaction = db.transaction(["saves_os"], "readonly");

    const objectStore = transaction.objectStore("saves_os");

    const getRequest = objectStore.get(name);

    // Report on the success of the transaction completing, when everything is done
    transaction.addEventListener("complete", () => {
        console.log("Transaction completed: save retrieved.");
        gameboy.load_save(getRequest.result.ram);
    });

    transaction.addEventListener("error", (e) => {
        console.log(e)
        console.log("Transaction not opened due to error")
    });
}

export {save, load};