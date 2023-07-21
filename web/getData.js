async function getData() {
    let data = await fetch("/api/name/Zachary/Boehm?" + new URLSearchParams({
        name: "zachary",
        age: 23
    }), { 
        method: 'POST', 
        body: JSON.stringify({ 'age': 23, 'male': true }),
        headers: {
          'Content-Type': 'application/json; charset=UTF-8',
        },
    });

    if ( data.ok ) {
        let userData = await data.json();
        console.log(userData);
    } else {
        console.log(data);
    }
}

async function getPlain() {
    let data = await fetch("/api/plain", { 
        method: 'POST', 
        body: 'Some other data',
        headers: {
          'Content-Type': 'text/plain; charset=UTF-8',
        },
    });

    if ( data.ok ) {
        let res = await data.text();
        console.log(res);
    } else {
        console.log(data);
    }
}
