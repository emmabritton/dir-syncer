## Directory Syncer

Synchronises file contents from one directory to another

You should probably use `rsync` or `git` instead of this.

### Usage
```
Synchronises file contents from one directory to another. Actions sync'd are adding, modifying and deleting of files. 
Note that any files that match the include pattern(s) will be deleted from the destination directory if they are not
also in the source directory.
Note this is not recursive and all files beginning a period/full stop are ignored.

Generally best to run with -v so a record of files changed is generated.

USAGE:
    dirsync [FLAGS] [OPTIONS] --dest_dir <DIR> --source_dir <DIR>

FLAGS:
    -c, --check      
            If set then this program will just print out the operations that it would perform and exits

    -h, --help       
            Prints help information

    -V, --version    
            Prints version information

    -v, --verbose    
            Set verbosity of program (between 0 and 3)


OPTIONS:
    -d, --dest_dir <DIR>         
            The directory to sync actions to

    -e, --exclude <REGEX>...     
            Regex for file name (including extension) to ignored when sync'ing

    -f, --freq <MINUTES>         
            How frequently (in minutes) this program check attempt to sync the directories, the timer is started once
            the last operation from the previous check completes. [default: 5]
    -i, --include <REGEX>...     
            Regex for file name (including extension) to be sync'd [default: .*]

    -o, --operations <NUMBER>    
            If set to 1 then this program will only add, modify or delete one file per check [default: 1]

    -s, --source_dir <DIR>       
            The directory to sync actions from
```

### License
```
Copyright 2020 Ray Britton

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
