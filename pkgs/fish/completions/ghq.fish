# The MIT License (MIT)
# 
# Copyright (c) 2016 Decors
# 
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
# 
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
# 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

function __fish_ghq_needs_command
    set cmd (commandline -opc)
    if [ (count $cmd) -eq 1 -a $cmd[1] = "ghq" ]
        return 0
    end
    return 1
end

function __fish_ghq_using_command
    set cmd (commandline -opc)
    if [ (count $cmd) -gt 1 ]
        if [ $argv[1] = $cmd[2] ]
            return 0
        end
    end
    return 1
end

# Help
function __fish_ghq_help_topics
    for c in get list root create
        printf "%s\thelp topic\n" $c
    end
end

function __fish_ghq_vcs
    for c in git subversion git-svn mercurial darcs
        printf "%s\tVCS\n" $c
    end
    printf "github\tAlias for git\n"
    printf "svn\tAlias for subversion\n"
    printf "hg\tAlias for mercurial\n"
end

complete -f -c ghq -n "__fish_ghq_needs_command" -a help -d "Shows a list of commands or help for one command"
complete -f -c ghq -n "__fish_ghq_using_command help" -a "(__fish_ghq_help_topics)"

complete -f -c ghq -n "__fish_ghq_needs_command" -a get -d "Clone/sync with a remote repository"
complete -f -c ghq -n "__fish_ghq_using_command get" -l update -s u -d "Update local repository if cloned already"
complete -f -c ghq -n "__fish_ghq_using_command get" -s p -d "Clone with SSH"
complete -f -c ghq -n "__fish_ghq_using_command get" -l shallow -d "Do a shallow clone"
complete -f -c ghq -n "__fish_ghq_using_command get" -l look -s l -d "Look after get"
complete -f -c ghq -n "__fish_ghq_using_command get" -l vcs -d "Specify VCS backend for cloning" -r -a "(__fish_ghq_vcs)"
complete -f -c ghq -n "__fish_ghq_using_command get" -l silent -s s -d "Clone or update silently"
complete -f -c ghq -n "__fish_ghq_using_command get" -l no-recursive -d "Prevent recursive fetching"
complete -f -c ghq -n "__fish_ghq_using_command get" -l branch -s b -d "Specify branch name. This flag implies --single-branch on Git"
complete -f -c ghq -n "__fish_ghq_using_command get" -l parallel -s P -d "Import parallely"
complete -f -c ghq -n "__fish_ghq_using_command get" -l help -s h -d "Show help"

complete -f -c ghq -n "__fish_ghq_needs_command" -a list -d "List local repositories"
complete -f -c ghq -n "__fish_ghq_using_command list" -l exact -s e -d "Perform an exact match"
complete -f -c ghq -n "__fish_ghq_using_command list" -l vcs -d "Specify VCS backend for matching" -r -a "(__fish_ghq_vcs)"
complete -f -c ghq -n "__fish_ghq_using_command list" -l full-path -s p -d "Print full paths"
complete -f -c ghq -n "__fish_ghq_using_command list" -l unique -d "Print unique subpaths"
complete -f -c ghq -n "__fish_ghq_using_command list" -l help -s h -d "Show help"

complete -f -c ghq -n "__fish_ghq_needs_command" -a root -d "Show repositories' root"
complete -f -c ghq -n "__fish_ghq_using_command root" -l all -d "Show all roots"
complete -f -c ghq -n "__fish_ghq_using_command root" -l help -s h -d "Show help"

complete -f -c ghq -n "__fish_ghq_needs_command" -a create -d "Create a new repository"
complete -f -c ghq -n "__fish_ghq_using_command create" -l vcs -d "Specify VCS backend explicitly" -r -a "(__fish_ghq_vcs)"
complete -f -c ghq -n "__fish_ghq_using_command create" -l help -s h -d "Show help"

complete -f -c ghq -n "__fish_ghq_needs_command" -l help -s h -d "Show help"
complete -f -c ghq -n "__fish_ghq_needs_command" -l version -s v -d "Print the version"
