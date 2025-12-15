# rust-vim
# Final Report
## Members
| Name | Student Number | Email |
|-|-|-|
| Ray Dai | 1006473086 | ray.dai@mail.utoronto.ca|
| James He | 1004118171 | james.he@mail.utoronto.ca|

## Motivation
In the software engineering community, Vim is a popular choice of code editor thanks to its efficiency. The editor would be even more appealing and powerful if users can benefit from large-language models at the same time. An existing solution is Neovim, however Neovim comes packaged with excessive additional packages and extensions, which the typical user may rarely use.

To our knowledge, there does not exist a lightweight VIM-styled CLI text editor with Language-Server-Protocol (LSP) and Copilot-style LLM integration for code autocompletion. Our project aims to address this gap. We aim to first develop a CLI text editor in Rust. Then, if time allows, we will implement LSP and LLM support and demonstrate popular features such as code autocompletion.

We believe implementing Vim in Rust is an ideal course project as there is a substantial opportunity for learning, there are many existing implementations for reference or comparison, and we are interested in deepening our understanding of Vim and its intricacies.

The technical complexity of this project is substantial enough to provide valuable learning experiences. The core challenges of the project align with Rust’s primary strengths. For example, Vim uses buffers to store large text files, and these buffers must be managed carefully to ensure data validity, and to prevent issues such as data corruption, duplication and unwanted mutations when interacting with the filesystem and terminal inputs. For our implementation, the Ropey Rust crate (https://crates.io/crates/ropey) happens to provide an efficient way to manage these text buffers.

The presence of reference implementations helps narrow the scope of our work to a manageable amount within the limited timeframe. This course focuses on learning the Rust language and emphasises implementation rather than design.  Given Vim is a well-established software system, we can refer to its existing architecture and focus on the implementation details, which is the focus of the course.

Lastly, by developing an implementation of Vim, we naturally become better Vim users ourselves. Since Vim is known to boost productivity for software engineers, learning its intricacies provides clear benefits. In fact, one of our teammates just learnt to use hjkl rather than the arrow keys to navigate in the Normal Mode during our first project meeting!
## Objectives
The key objective of this project is to develop a CLI text editor. This means that the bare minimum features that must be implemented include a Terminal UI, the ability to manipulate (i.e. open, read, write, and close) text files, and the ability to read user input for editing text files. These are fundamental features that are required to establish bare minimum functionality of our text editor.

Side objectives include implementing key features inspired by vim. This includes implementing the following features, which are described in more detail in the Key Features section
* A state machine for different modes of operation, including Normal Mode, Insert Mode, Command Mode, and a new Search Mode (separated from vim’s Command Mode)
* A status bar at the bottom displaying the cursor’s location in the file, as well as user input and feedback messages for the user to read
* Text soft-wrapping so that lines of text which exceed the width of the terminal window will be automatically wrapped around to a new line without inserting newline characters

Finally, there were two extra bonus objectives of this project. The first one is to implement Language Server Protocol (LSP) support for different coding languages, to allow for features such as keyword coloring, jump to definition, and even rudimentary code completion based on recent text edits. The second is to support LLM integration, allowing users to prompt a supported LLM platform to generate a proposed set of edits based on the user’s prompt and text file contents, which the user can then accept or reject. Unfortunately due to time constraints, these objectives were not completed, however, there are plans for potential future implementation using rust crates similar to llm-lsp (https://crates.io/crates/llm-lsp).
## Key Features
As a CLI text editor, the following core features are supported for basic text editing functionality
* The ability to both open and read existing files, as well as create new ones if the provided file name does not already exist
* A cursor that the user can manipulate using arrow keys, in order to show where character insertion/deletion will take place.
    * The cursor is bound to regions of the terminal that correspond to the file's contents, even if the terminal window itself is shrunk by the user
        * Shrinking the terminal window will push the cursor upwards and/or to the left as needed to ensure it stays in a legal position for indexing file contents
    * The cursor snaps to the end of a shorter line when moving up/down between lines, rather than illegally occupy the empty space which does not correspond to file contents
    * The cursor ‘slips’ through extra columns of a wide character (notably emoticons and tab spaces) to prevent the cursor from illegally occupying the middle of a wide character, causing the editor to incorrectly index file contents.
    * The cursor changes shape in Insert Mode to allow users to add text to the beginning or end of a line.
* The ability to scroll through lines of the text file if it exceeds the height of the terminal window.
    * File contents start scrolling upwards when the cursor reaches the bottom of the terminal window, and scroll downwards when the cursor reaches the top of the terminal window
    * Scrolling stops once the beginning and/or end of the file is reached, and briefly displays a message to the user if further scrolling is attempted
* The ability to adapt to different terminal shapes and sizes, even if the user changes it during runtime.

Since this is a vim-inspired text editor, vim’s most important modes are also implemented to include the following:
* Normal Mode for viewing
    * This is a heavily-stripped down version of vim’s Normal Mode, serving as a “hub” for transitioning between other modes, or otherwise just navigating the cursor and viewing file contents using the arrow keys.
* Insert Mode for text editing
* Command Mode for entering commands. The following commands are implemented
    * Write (:w)
    * Quit (:q)
    * Write-Quit (:wq)
    * Line Number Toggling (:num)
    * Line Deletion (:dd)
* Search Mode for searching strings
    * Unlike vim, this was separated from the Command Mode for better code structuring and ease of use
    * If matches exist, they will then be highlighted until the user hits [Esc] in Normal Mode or begins a new search
    * Supports Regex searching

Hotkeys and navigation between modes is discussed in the user guide below. Other vim features also implemented in this app include a status bar at the bottom of the terminal window, and text soft-wrapping.

The status bar acts as a fundamental part of our app’s ui, as it displays key information such as the cursor’s location in the file, the current mode and important hot-keys, feedback/error messages, and the user input for commands and search queries. The cursor’s location is represented as a row, and a column, which corresponds to which line of the file, and which column of the file the cursor is currently located at.

Text soft-wrapping allows rust-vim to display lines of text which exceed the width of the terminal window, by automatically wrapping them around to a new line, without inserting extra newline characters. Our implementation of text wrapping is also robust enough to adapt to different terminal sizes, even if the user resizes the terminal during runtime. Text wrapping was implemented to help improve user experience, as the alternative would be to have long lines go out of the terminal window and add horizontal scrolling. By wrapping text instead, the true contents of the file are made more clear to the user, and there is less ambiguity about whether the line extends past the terminal window. For users that may prefer horizontal scrolling over text wrapping, a command to switch between wrapping text and horizontal scrolling may be implemented in the future.

Due to time constraints, the following vim modes, commands, and features will not be supported,
* Visual Mode Highlighting
    * Copy-Paste
    * Mass-delete
* Command sequencing in Normal Mode
* Binary, Org, and Replace modes
* Text undo/redo functionality

Finally, a quick-help pop-up was also implemented to give users a reference manual for how to use the commands and hotkeys provided. Users can view it from the Normal Mode by tapping [z], scroll through it with up/down arrow keys, and close it using [Esc].
## User Guide
This section acts as a brief user guide of how to use the different features rust-vim provides. For instructions on how to compile, and then run rust-vim to open or create a file, please refer to the Reproducibility Guide.
### Hotkey Summary
The Help Pop-up below shows a brief overall summary of the important hotkeys in each mode.
[TODO: INSERT IMAGE]

Detailed explanations of each mode are provided in the corresponding sections below
### Normal Mode
On startup, rust-vim will use the calling terminal window to display the contents of the file in Normal Mode, with the cursor starting in the top left of the screen at the first column of the first line of text. The Normal Mode is used for viewing the contents of the file, and to act as a ‘hub’ between the other modes. Users can move the cursor over file contents via the arrow keys. Moving the cursor to the top or bottom of the terminal window and continuing to move it up or down will cause the content in the terminal window to scroll, allowing users to view off-screen content if there is any. Scrolling stops once the beginning or end of the file is reached.

The following hotkeys are used to navigate to other modes from Normal Mode:
[i] to enter Insert Mode and start editing the file contents
[:] to enter Command Mode and start writing commands
[/] to enter Search Mode and start writing a search query

Additionally, the user can also open up the quick-help pop-up using the [z] hotkey, to view a quick-reference user manual. The help pop-up’s contents can be scrolled using the up/down arrow keys, and users can return to Normal Mode using the [Esc] key.
### Insert Mode
In Insertion Mode, the user can still move the cursor with the arrow keys just like in Normal Mode. To edit the file, simply type on the keyboard to insert characters to the right of the cursor’s current location. The cursor will then automatically move rightwards with whatever was typed, just like in traditional text editors. Use the [Enter] key to insert a new line, and the [Backspace] key to remove the character to the left of the cursor’s current location.

To exit Insert Mode and return to Normal Mode, simply tap the [Esc] key.
### Command Mode
In Command Mode, the cursor is locked, and users can no longer move it with arrow keys. Instead, users can type in desired commands to be executed. rust-vim automatically records the user’s keystrokes, and displays them for reference in the status bar below the file contents.

Users can use the [Backspace] key to delete the right-most character of the command being typed, in case they make a mistake. Deleting all characters in this manner (including the [:] character used to enter Command Mode) will return users back to Normal Mode. Users can also use the [Esc] key to exit Command Mode prematurely without submitting a command, returning them back to Normal Mode.

Once the user has typed out a desired command to run, users can press the [Enter] key to submit and run the command, before returning to Normal Mode if the file was not closed. Implemented commands are shown below:
* [:w] to write and save over the file without quitting rust-vim
* [:q] to terminate rust-vim without writing to the file, and then restore the terminal window to its previous state before starting rust-vim
    * This command will first display a pop-up window asking the user to confirm their intention to quit without saving
    * Options can be selected with left/right arrow keys
    * Selected option can be confirmed using [Enter] key
    * Users can also hit [Esc] to cancel the command, closing the pop-up and returning them to Normal Mode
* [:wq] to write and save over the file, terminate rust-vim, and then restore the terminal window to its previous state before starting rust-vim
* [:num] to toggle whether rust-vim should also display line numbers to the left of the file contents
* [:dd] to delete the current file line at the cursor

If the submitted command does not match any of the above, the user is returned to the Normal Mode with an error message shown in the status bar informing the user that their command was invalid. This error message goes away after any user input is received.

### Search Mode
In Search Mode, the cursor is locked, and users can no longer move it with arrow keys. Instead, users can type in the desired string to be queried for in the file. rust-vim automatically records the user’s keystrokes, and displays them for reference in the status bar below the file contents. rust-vim supports Regex searching.

Users can use the [Backspace] key to delete the right-most character of the command being typed, in case they make a mistake. Deleting all characters in this manner (including the [/] character used to enter Search Mode) will return users back to Normal Mode. Users can also use the [Esc] key to exit Search Mode prematurely without querying anything, returning them back to Normal Mode.

Once a user has finished typing the string they wish to search for, they can submit the query using the [Enter] key. If matches are found, rust-vim will automatically highlight them and return the user to Normal Mode. Search highlights will persist until the user hits [Esc] in Normal Mode, or until the user begins a new search query. 

If rust-vim does not find any matches for the submitted query in the file, users are returned to Normal Mode with an error message indicating this result.
## Reproducibility Guide
To build rust-vim, first ensure that cargo has been installed on the system. You can install cargo by following these instructions here: https://doc.rust-lang.org/cargo/getting-started/installation.html

Next, download this entire github repo as a zip file, and extract it. After extraction, you should have the src/ folder, as well as the Cargo.lock and Cargo.toml files together in the same folder. The path to this folder will be referred to as <extraction_path>.

To build the binary file, navigate to <extraction_path> in a terminal window, and run the following command

	cargo build --release

This will build the release version of rust-vim, located at

    <extracted_path>/target/release/rust-vim

After building, feel free to relocate the rust-vim binary file to any location of your choosing, as long as you keep track of where it is. Once you’ve decided on a location to keep rust-vim, we recommend adding the full path to that location to your device’s PATH environment variable for ease of use. Instructions on how to do so can be found here below for Windows, Linux, and MacOS

* Windows: https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/
* Linux: https://www.baeldung.com/linux/path-variable
* MacOS: https://medium.com/@B-Treftz/macos-adding-a-directory-to-your-path-fe7f19edd2f7

Once installed, you can run rust-vim by simply running the following command in a terminal

	<path_to_rust-vim>/rust-vim <file_name>

Which will open the file with the name <file_name> if it exists, or create it if it doesn’t.

If you added rust-vim’s location to your device’s PATH environment variable, you can instead simply use

	rust-vim <file_name>

rust-vim also works with paths to files that are not in the current directory. For example, to open a file named rustacean.txt located in the ./rust_is_cool directory, you can use the following command

	rust-vim rust_is_cool/rustacean.txt


## Contributions
The team members developed the core features collaboratively. Ray primarily focused on a user interface that closely mimics Vim. James focused on the backend that efficiently stores and retrieves the text. The interaction between the front-end and backend was a joint effort between James and Ray.
| Name | Task | Other Notes |
|-|-|-|
|Ray|Terminal UI Setup|<ul><li>Clear the terminal and display the application</li><li>Restore the terminal on app closure</li><li>Accept user inputs</li><li>Handle the user resizing terminal window without crashing</li></ul> |
|Ray|Status/Message Bar|<ul><li>Display current mode</li><li>Display error messages</li><li>Display user input</li><li>Display cursor infile location</li></ul> |
|James|Read from file system to buffer|<ul><li>Load file into the backend model</li><li>Create an empty file if the file does not already exist</li></ul>|
|James|Write to file system|<ul><li>Serialize the backend model into a file for persistent storage</li></ul>|
|Ray|Text Wrapping|<ul><li>Must adapt to terminal resizing</li></ul>|
|Ray|Cursor Positioning, Tracking, and Movement|<ul><li>Cursor must be bound to file contents and terminal window</li><li>Cursor's infile location must be tracked for backend use</li><li>Snap cursor to end of line when needed</li><li>Slip cursor through wide characters</li><li>Must adapt to user shrinking terminal window by pushing cursor up/left as needed</li></ul>|
|Ray|Scroll File Contents||
|James|Insert content to buffer|<ul><li>Handle inputs from the UI and record them properly in the backend model</li></ul>|
|James|Delete content from buffer|<ul><li> Handle delete requests from the UI</li><li>Translate display cursor location into character index in the backend model</li><li>Delete the text at the corresponding location</li></ul>|
|Ray|Controller State Machine for Vim Modes|<ul><li>Some features were implemented as extra modes under the hood</li></ul>|
|James|Search for content in buffer| <ul><li>Traverse the backend data model to return matches.</li></ul>|
|Ray|Highlight Matches to Search Queries||
|Ray|Implement Line Number Toggle & Delete Line Commands|<ul><li>Shift file contents and cursor as needed to adapt to added UI elements</li></ul>|
|Ray|Help and Confirm Quit Pop-up Window||

## Lessons Learned and Concluding Remarks
Two key lessons were learned from this project.

For one, we learned firsthand about the many struggles that can arise when co-operatively developing code without proper project management. In a professional environment, we would first design an agreed upon blueprint for the overall project as a first step, including overall architecture, skeleton code, and API definitions between key components, before individually working on assigned parts. Unfortunately, due to our busy schedules, we were unable to meet up and create this blueprint, instead skipping this crucial first step and jumping straight into writing code. As a result, our code does not follow the proposed MVC architecture very strictly, which may result in problems such as code readability, maintainability, scalability, and potentially even performance drawbacks.

One key example of this is the display_content which was stored in the App struct in the controller. The display_content field of the App struct is meant to store the wrapped lines of text (along with metadata like line number) for the View to display. According to the MVC architecture, the display_content would be more appropriately located in either the View (since the wrapped lines would be directly displayed) or the Model (since the original lines of text should be managed by the Model). Instead, the current implementation was built based on a tutorial for the ratatui crate (https://ratatui.rs/tutorials/json-editor/app/) which uses a different paradigm, where the entire App state is managed in one struct (which we placed in the controller to manage other state variables such as mode), while another file handles the actual display in the terminal. As a result, there was some confusion on implementation when passing the project between hands.

Another key lesson learned from this project was the complexity that can go into developing effective text writers, and the importance of good base algorithms over implementation fixes. One example was figuring out how to manage the cursor’s location in the file. A good algorithm is capable of handling many edge cases without requiring too many fixes for unhandled edge cases. In comparison, our algorithm to calculate and track the cursor's position in the file resulted in many unexpected complications arising such as the presence of wide characters, which if improperly handled, could result in the cursor entering an illegal position in the middle of the character.

Tracking the cursor was also difficult because of the text wrapping, which meant that one line in the file could actually be represented by multiple lines on the terminal window. This meant that tracking things such as the cursor’s infile line index, and the cursor’s infile column number, couldn’t be done by simply using the cursor’s coordinates in the terminal. Instead, relevant details for each displayed line had to be stored separately, and then looked up in order to calculate the cursor’s infile line index and infile column number. The requirement of handling terminal resizing and scrolling only made this task more complex, and added more edge cases to handle seperately. 

Due to these complexities and inefficiencies caused by poor project management, we were also unfortunately unable to complete the bonus objectives of implementing LSP and LLM support into rust-vim for the submission of this project. However, there were plans to do so using the llm-lsp rust crate (https://crates.io/crates/llm-lsp) or similar crates.

To conclude, while there is still plenty of work that could be done to improve this application, developing rusty-vim was still a very intellectually stimulating experience, which taught us many lessons about both technical details of developing an application in rust with the crates used, as well as the importance of proper project management and organization. The unexpected complexity and challenges faced in this project have also given us a much greater appreciation for all the different considerations that must be taken when developing something as seemingly simple as a CLI text editor.
