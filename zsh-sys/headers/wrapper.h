#ifndef have_zshQsmain_module
#define have_zshQsmain_module

# ifndef IMPORTING_MODULE_zshQsmain
#  ifndef MODULE
#   define boot_ boot_zshQsmain
#   define cleanup_ cleanup_zshQsmain
#   define features_ features_zshQsmain
#   define enables_ enables_zshQsmain
#   define setup_ setup_zshQsmain
#   define finish_ finish_zshQsmain
#  endif /* !MODULE */
# endif /* !IMPORTING_MODULE_zshQsmain */

/* Extra headers for this module */
# include "../config.h"
# include "zsh_system.h"
# include "zsh.h"
# include "sigcount.h"
# include "signals.h"
# include "prototypes.h"
# include "hashtable.h"
# include "ztype.h"

# undef mod_import_variable
# undef mod_import_function
# if defined(IMPORTING_MODULE_zshQsmain) &&  defined(MODULE)
#  define mod_import_variable 
#  define mod_import_function 
# else
#  define mod_import_function
#  define mod_import_variable
# endif /* IMPORTING_MODULE_zshQsmain && MODULE */
# include "builtin.epro"
# include "compat.epro"
# include "cond.epro"
# include "context.epro"
# include "exec.epro"
# include "glob.epro"
# include "hashtable.epro"
# include "hashnameddir.epro"
# include "hist.epro"
# include "init.epro"
# include "input.epro"
# include "jobs.epro"
# include "lex.epro"
# include "linklist.epro"
# include "loop.epro"
# include "math.epro"
# include "mem.epro"
# include "module.epro"
# include "options.epro"
# include "params.epro"
# include "parse.epro"
# include "pattern.epro"
# include "prompt.epro"
# include "signals.epro"
# include "signames.epro"
# include "sort.epro"
# include "string.epro"
# include "subst.epro"
# include "text.epro"
# include "utils.epro"
# include "openssh_bsd_setres_id.epro"
# undef mod_import_variable
# define mod_import_variable
# undef mod_import_variable
# define mod_import_variable
# ifndef mod_export
#  define mod_export 
# endif /* mod_export */

#endif /* !have_zshQsmain_module */
