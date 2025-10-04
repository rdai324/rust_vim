# rust_vim 

# Project Proposal

## Filling a Gap
In the software engineering community, Vim is a popular choice of code editor thanks to its efficiency. The editor would be even more appealing and powerful if users can benefit from large-language models at the same time. An existing solution is Neovim, however Neovim comes packaged with excessive additional packages and extensions, which the typical user may rarely use. 

To our knowledge, there does not exist a lightweight VIM-styled CLI text editor with Copilot-style LLM integration for code autocompletion. Our project aims to address this gap. We aim to first develop a CLI text editor in Rust. Then, if time allows, we will implement LLM support and demonstrate popular features such as code autocompletion.

## Motivation
We believe implementing Vim in Rust is an ideal course project as there is a substantial opportunity for learning, there are many existing implementations for reference or comparison, and we are interested in deepening our understanding of Vim and its intricacies.

The technical complexity of this project is substantial enough to provide valuable learning experiences. The core challenges of the project align with Rust’s primary strengths. For example, Vim uses buffers to store texts, and these buffers must be managed carefully to ensure data validity, and to prevent issues such as data corruption, duplication and unwanted mutations when interacting with the filesystem and terminal inputs. Additionally, implementing efficient searching in large files requires leveraging asynchronicity in Rust for speedy performance. 

The presence of reference implementations helps narrow the scope of our work to a manageable amount within the limited timeframe. This course focuses on learning the Rust language and emphasises implementation rather than design.  Given Vim is a well-established software system, we can refer to its existing architecture and focus on the implementation details, which is the focus of the course. 

Furthermore, we can benchmark our implementation against existing implementations in C or C++. By comparing the performance of our implementation against the standard version - using the same text input and operations on the same hardware platform, we can measure the execution time. If time allows, we can perform an in-depth analysis to root cause the difference - whether they come from the language itself, or algorithmic differences in our implementation. This profiling provides objective feedback on our work, which is crucial in real-world software engineering. 

Lastly, by developing an implementation of Vim, we naturally become better Vim users ourselves. Since Vim is known to boost productivity for software engineers, learning its intricacies provides clear benefits. In fact, one of our teammates just learnt to use hjkl rather than the arrow keys to navigate in the Normal Mode during our first project meeting!

## Objectives and Key Features
As the objective of this project is to develop a CLI text editor, the bare minimum features that must be implemented include a Terminal UI, the ability to manipulate (i.e. open, read, write, and close) text files, and the ability to read user input for editing text files. These are the fundamental features required to establish bare minimum functionality of our text editor. 

However, as we are developing a vim-inspired text editor, vim’s most important modes will also be implemented in a stripped-down fashion. 

The Insert mode will be fully implemented as our bare minimum text-editing features. 

The Normal mode and Visual mode will be heavily stripped down, with only HJKL movement being implemented for the Normal mode, and Copy-Paste functionality implemented for the Visual mode. 

Finally, the Command-line mode will be stripped down to the following most commonly used commands:
* Write, Quite, and Write-Quit
* Regex Searching
* Line Number Toggling

Due to time constraints, the following commands and features will not be supported
* Command sequencing in normal mode 
* Binary, Org, and Replace modes
* Text undo/redo functionality

Once the above desired features have been implemented, our text editor itself will be complete. If time allows, we will implement LLM-based code completion. 

## Tentative Plan
We plan on applying the Model-View-Controller architecture for this project. 

The Model component holds an in-memory representation of the text. Its major functionalities include file I/O, insertion, deletion, replacement, and search. 

The View component is responsible for displaying the right content on the screen given the cursor location and the buffer content.

The Controller module is primarily responsible for handling the user inputs, parsing commands, keeping track of the current mode, and coordinating with the Model component and View component.  

James will be responsible for the model component as he has prior work experience and expertise with buffer management. Ray will focus on the front-end tasks such as the UI and the input controller, due to his interests and experiences with front-end development from prior coursework. The following table shows a more detailed breakdown of tasks and their assignment. Component-API level unit testing will be taken care of by its respective owners. The final system integration will be a joint effort between James and Ray.

| Task                                                | Team Member Responsible                                       |
| --------------------------------------------------- | ------------------------------------------------------------- |
| Terminal UI Initialization                          | Ray                                                           |
| Caret positioning and movement                      | Ray                                                           |
| Status/Message Bar                                  | Ray                                                           |
| Read from file system to buffer                     | James                                                         |
| Searching for content in buffer                     | James                                                         |
| File content renderer                               | Ray                                                           |
| UI Scrolling                                        | Ray                                                           |
| Controller state machine for Vim Modes              | Ray                                                           |
| Implement line number toggle command                | Ray                                                           |
| Inserting content to the buffer                     | James                                                         |
| Write to file system                                | James                                                         |
| Deleting content from buffer                        | James                                                         |
| Copying content from buffer                         | James                                                         |
| Updating file renderer to respond to buffer changes | Ray                                                           |
| Add colors to file renderer                         | Ray                                                           |
| Implement Copilot LLM Support                       | To-be-determined based on the actual progress of the project. |
