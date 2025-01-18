# Downloads Sorter

Is your downloads folder constantly filled to the brim with files upon files, some having been there since the dawn of mankind?
This small program runs in the background taking virtually no footprint, automatically sorting all your files in different folders
in your downloads according to file type. Idea and testing by Mammutkung.

## Configuration

This can be configured with a small JSON configuration file in the program directory. 
An example configuration looks like this:
```json
{
	"Documents": ["txt", "pdf", "docx", "xlsx", "odt"],
	"Archives": ["zip", "rar", "7z"],
	"Executables": ["exe", "msi"],
	"Images": ["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp"],
	"Videos": ["mp4", "mov", "avi", "webm", "mkv", "wmv"],
	"Audio": ["mp3", "wav", "ogg", "flac", "m4a"]
}
```

## How it works

The sorter works by reading in your configuration, watching your downloads folder for any file creation, 
and moving files to their respective folders whenever a change has been detected. When a file is being downloaded,
the sorter ignores the creation of any temporary files like .tmp and .crdownload on Windows to ensure your download 
is not interrupted. The downloaded file will thereafter be moved into its respective folder. If the files type is 
not specified in configuration, it will be moved into a folder labelled 'Other'.

Typically, this kind of automation would be written in a small and simple language like Python - but I 
decided to write this in Rust both to minimise the performance hit that the watching and sorting process would incur,
and also to avoid the need for any other external dependencies to be installed. As such, in my testing, it has 
virtually no memory or CPU footprint. I measured at maximum 1.1MB of memory usage, and negligable CPU usage.

Once the program is running, you won't notice its existence. You can terminate it by finding and killing the process.
On Windows, this will be done in Task Manager.

## Cross-platform Capabilities

I have only tested this on Windows, however, it should theoretically be able to run on Linux and MacOS.

