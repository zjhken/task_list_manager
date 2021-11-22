# task_list_manager

The backend is written in Rust.
The key component of the backend is my self-written Rust web framework called "Seafloor".
The motivation is to create a  framework with more elegant APIs.

## logging
currently I didn't implement any logging mechanism in it, all print to the stdout.
User can redirect the stdout to any place they want.
The way of stdout better fit the docer way.

## storage
My original thinking is store the data to SQLite, but in this way there will be a file generated in local disk.
To make it simpler, I put the data in BTreeMap.

# How to build
Rust program need to be complied.
So firstly need to setup the Rust environment in the official standard way.

If you want to run it internally, there are guides for you can refer to.
I will not paste the link here to avoid some keyword.

# How to run.
Simply run the binary in local.
```bash
nohup ./task_list_manager > operation.log 2>&1 &
```
I will build a binary for you for the backend.
And I've prepared a script for you to run in /src/bin/task.sh
run `./src/bin/tasks.sh -h` , it will show you how to use.