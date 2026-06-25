#!/bin/sh
# This script generates the latexdiff between two git revisions
#
# It uses a detached git worktrees to compare the two revisions
# By default, the older revision is HEAD~1 and the newer revision is HEAD unless
# you set a ldiff.ref file with the older revision number to use as the reference.
#
# Tectonic is used as the TeX compiler by default.
#
# You can set environment variables to override the default behavior, e.g.:
#
# TEXCOMPILER_CMD=tectonic
# DIFF_BIN=rust-latexdiff
# DIFF_CMD="${DIFF_BIN} diff"
# EXPAND_CMD="${DIFF_BIN} expand"
#

# TeX compiler command
TEXCOMPILER_CMD=${TEXCOMPILER_CMD:="tectonic"}
# Commands for expanding and diffing
DIFF_BIN=${DIFF_BIN="$(which rust-latexdiff)"}

# Resolve the absolute path of the diff binary
DIFF_BIN=$(readlink -f "${DIFF_BIN}")
# specific commands for expanding and diffing
EXPAND_CMD=${EXPAND_CMD="${DIFF_BIN} expand"}
DIFF_CMD=${DIFF_CMD="${DIFF_BIN} diff"}

Help()
{
   # Display Help
   echo "Generates the latexdiff between two git revisions"
   echo
   echo "Syntax: $0 [-p|n|h] main_document_filename"
   echo "options:"
   echo "p     older version (default HEAD~1)"
   echo "n     newer version (default HEAD)"
   echo "o     output document name (default: diff_output.pdf)"
   echo "h     Print this Help."
   echo
   echo " example: $0 -p HEAD~3 main"
   echo
}

[ -f ldiff.ref ] && oldrev=$(cat ldiff.ref) || oldrev=HEAD~1
newrev=HEAD
outputname=diff_output


# Get the options
while getopts ":hp:n:o:" option; do
   case $option in
      p)
	 oldrev=${OPTARG}
	 ;;
      n)
	 newrev=${OPTARG}
	 ;;
      o)
	 outputname=${OPTARG}
	 ;;
      h|help) # display Help
         Help
         exit 0
	 ;;
      \?) # incorrect option
	 echo "Error: Invalid option" 1>&2
	 exit 1
	 ;;
   esac
done
shift $((OPTIND -1))

if [ $# -ne 1 ]; then
    echo "❌ illegal number of parameters" 1>&2

    Help
    exit 1
fi

! [ -f  ${DIFF_BIN} ] && echo "❌ ${DIFF_BIN} not found" 1>&2 && exit 1


# Directories for the two revisions
CWD=$(pwd)
TMPDIR=$(mktemp -d /tmp/git-latexdiff.XXXXXX)

# create temporary worktrees for the two revisions
echo "📂 creating temporary (detached) worktrees for the two revisions"
OLDREV_DIR="${TMPDIR}/start"
NEWREV_DIR="${TMPDIR}/end"

if [ -d "${OLDREV_DIR}" ]; then
    echo "⚠️ older version directory ${OLDREV_DIR} already exists" 1>&2
else
    git worktree add --detach ${OLDREV_DIR} ${oldrev}
fi
if [ -d "${NEWREV_DIR}" ]; then
    echo "⚠️ newer version directory ${NEWREV_DIR} already exists" 1>&2
else
    git worktree add --detach ${NEWREV_DIR} ${newrev}
fi

# checking main document
MAINDOC=${1%.*}
if ! [ -f "${OLDREV_DIR}/${MAINDOC}.tex" ]; then
    echo "Main document ${MAINDOC}.tex not found in the older version" 1>&2
    exit 1
fi
if ! [ -f "${NEWREV_DIR}/${MAINDOC}.tex" ]; then
    echo "Main document ${MAINDOC}.tex not found in the newer version" 1>&2
    exit 1
fi

# Output document name
OUTPUT_DOC="${NEWREV_DIR}/${outputname%.*}.tex"
OUTPUT_PDF="${OUTPUT_DOC%.tex}.pdf"
# move the final pdf outside the worktree to CWD
DEST_PDF="${OUTPUT_PDF##*/}"

echo "Generating latexdiff document between rev:${oldrev} and rev:${newrev}"
echo ""
echo "       temporary directory: ${TMPDIR}"
echo "         main tex document: ${MAINDOC}.tex"
echo " temporary output document: ${OUTPUT_DOC}"
echo "              final output: ${CWD}/${DEST_PDF}"
echo ""
echo "Using:"
echo "   diff command: ${DIFF_CMD}"
echo "   expand command: ${EXPAND_CMD}"
echo "   latex compiler: ${TEXCOMPILER_CMD}"
echo ""

# step 1: flatten the files to make sure \input and \include are injected properly
echo "📄 flattening the main tex document to inject \input and \include"
cd "${OLDREV_DIR}" && ${EXPAND_CMD} ${MAINDOC}.tex -o ${MAINDOC}_flat.tex
cd "${NEWREV_DIR}" && ${EXPAND_CMD} ${MAINDOC}.tex -o ${MAINDOC}_flat.tex

# step 2: run latexdiff on the flattened files
echo "🔍 running latexdiff on the flattened files"
cd ${CWD}
${DIFF_CMD} -o ${OUTPUT_DOC} ${OLDREV_DIR}/${MAINDOC}_flat.tex ${NEWREV_DIR}/${MAINDOC}_flat.tex || exit 1

# step 3: compile the output document
echo "📄 compiling the output document"
cd "${NEWREV_DIR}"
${TEXCOMPILER_CMD} ${OUTPUT_DOC} || exit 1

# step 4: move the final PDF to the CWD
echo "📄 moving final PDF to ${CWD}/${DEST_PDF}"
mv ${OUTPUT_PDF} ${CWD}/${DEST_PDF} || exit 1

# step 5: cleanup
# remove the temporary worktree link and directories
echo "🧹 cleaning up"
git worktree remove -f ${OLDREV_DIR} || exit 1
git worktree remove -f ${NEWREV_DIR} || exit 1
rm -rf "${TMPDIR}"

echo "✅ done. output diff document: ${CWD}/${DEST_PDF}"
