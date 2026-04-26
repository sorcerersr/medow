# Download

Implement the Download-Feature.

- the selection of a search result (checkbox) in the search view should be remembered across all pages on page switches
- add two new buttons below the search results in search view: "download" and "reset"
- the buttons should be aligned to the right
- the download button starts the download of all selected items (there should be a url to download that points to a mp4 file or a m3u8 playlist)
- when the download start is trigged switch to the downloads view
- the download view should show the status of the downloads for each file to download
- the "reset" button resets all selections across all pages
- the downloads must be non blocking / async
- destination of downloads is the download path from the config
- as the destination filename use the "title" with spaces replaced with underscore and the file extension of the original file.
