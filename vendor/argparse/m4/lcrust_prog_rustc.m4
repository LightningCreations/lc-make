# Copyright 2021 Connor Horman

# Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

# The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

AC_DEFUN([LCRUST_PROG_RUSTC],[
    AC_REQUIRE([AC_PROG_CC])
    AC_REQUIRE([AC_CANONICAL_HOST])
    AC_ARG_VAR(RUSTC,[Rust compiler to use])
    AC_ARG_VAR(RUSTFLAGS,[Flags to pass to the rust compiler])

    if test "$RUSTFLAGS" \= "" 
    then
        RUSTFLAGS="-C opt-level=2 -g"
    fi

    if test x$host_alias != x 
    then
        AC_PATH_PROGS(RUSTC,[rustc lcrustc $host-gccrs])
    else 
        AC_PATH_PROGS(RUSTC,[rustc lcrustc $host-gccrs gccrs])
    fi

    if test "$RUSTC" \= ""
    then
        AC_MSG_ERROR([Failed to find a rust compiler. Install rustc in PATH, or set RUSTC to a suitable compiler])
    fi
    AC_MSG_CHECKING([how to compile for $host with $RUSTC])
    case x$RUSTC in 
        x${host_alias}-* | x*[\\/]${host_alias}-* )
            rustc_host_target=${host_alias}
            AC_MSG_RESULT([Not needed])
            ;;
        x${host}-* | x*[\\/]${host}-* )
            rustc_host_target=${host}
            AC_MSG_RESULT([Not needed])
            ;;

        x* )
            SAVE_RUSTFLAGS="$RUSTFLAGS"
            if test x$host_alias != x 
            then
                RUSTFLAGS="$RUSTFLAGS --target $host_alias"
                echo '#![no_std]' > comptest.rs 
                echo Trying target $host_alias >> config.log
                echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                if test $? -eq 0
                then
                    rustc_host_target=$host_alias
                else
                    echo Using target $host_alias failed >> config.log
                fi 
            fi
            
            if test x$rustc_host_target \= x
            then 
                RUSTFLAGS="$SAVE_RUSTFLAGS --target $host"
                echo '#![no_std]' > comptest.rs 
                echo Trying target $host >> config.log
                echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                if test $? -eq 0
                then
                    rustc_host_target=$host
                else
                    echo Using target $host failed >> config.log
                fi 
            fi

            if test x$rustc_host_target \= x
            then
                case $host in
                x86_64-*-mingw32 | i*86-*-mingw32 )
                    IFS="-" read rustc_host_arch rustc_host_vendor rustc_host_sys <<< "$host"
                    RUSTFLAGS="$SAVE_RUSTFLAGS --target ${rustc_host_arch}-pc-windows-gnu"
                    echo '#![no_std]' > comptest.rs 
                    echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                    $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_host_target=${rustc_host_arch}-pc-windows-gnu
                    fi
                    ;;
                x86_64-*-*-* | i*86-*-*-* )
                    IFS="-" read rustc_host_arch rustc_host_vendor rustc_host_os rustc_host_env <<< "$host"
                    RUSTFLAGS="$SAVE_RUSTFLAGS --target ${rustc_host_arch}-unknown-${rustc_host_os}-${rustc_host_env}"
                    echo '#![no_std]' > comptest.rs 
                    echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                    $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_host_target=${rustc_host_arch}-unknown-${rustc_host_os}-${rustc_host_env}
                    fi 
                    ;;
                x86_64-*-* | i*86-*-* )
                    IFS="-" read rustc_host_arch rustc_host_vendor rustc_host_sys <<< "$host"
                    RUSTFLAGS="$SAVE_RUSTFLAGS --target ${rustc_host_arch}-unknown-${rustc_host_sys}"
                    echo '#![no_std]' > comptest.rs 

                    echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                    $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_host_target=${rustc_host_arch}-unknown-${rustc_host_sys}
                    fi 
                    ;;
                *-*-* )
                    IFS="-" read rustc_host_arch rustc_host_vendor rustc_host_sys <<< "$host"
                    RUSTFLAGS="$SAVE_RUSTFLAGS --target ${rustc_host_arch}-${rustc_host_sys}"
                    echo '#![no_std]' > comptest.rs 

                    echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                    $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_host_target=${rustc_host_arch}-${rustc_host_sys}
                    fi
                    ;;
                esac
            fi
            if test x$rustc_host_target \= x
            then
                AC_MSG_RESULT([not found])
                AC_MSG_ERROR([Cannot compile to $host with $RUSTC])
            else
                AC_MSG_RESULT([--target $rustc_host_target])
            fi
        ;;
    esac
    rm -f comptest.rs libcomptest.rlib
    AC_MSG_CHECKING([whether Rust compiler works])
    echo 'fn main(){}' > comptest.rs 
    $RUSTC $RUSTFLAGS --crate-type bin --crate-name comptest comptest.rs 2>> config.log > /dev/null
    if test $? -ne 0
    then
        echo '#![no_std]' > comptest.rs
        $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest --emit link=libcomptest.rlib comptest.rs 2>> config.log > /dev/null
        if test $? -ne 0
        then
            AC_MSG_RESULT([no])
            AC_MSG_ERROR([Cannot compile a simple program with $RUSTC])
        else
            rustc_has_std=no
        fi
    else
        rustc_has_std=yes
    fi

    if test x$host_alias \= x 
    then
        ./comptest${EXEEXT}
        if test $? -ne 0
        then
            AC_MSG_RESULT([no])
            AC_MSG_ERROR([Cannot run executables compiled by $RUSTC])
        fi
    fi

    rm -f comptest.rs comptest${EXEEXT}

    AC_MSG_RESULT([yes])

    AC_SUBST(rustc_has_std)
    AC_SUBST(RUSTC)
    AC_SUBST(RUSTFLAGS)
])

AC_DEFUN([LCRUST_RUSTC_VERSION],[
    AC_REQUIRE([LCRUST_PROG_RUSTC])

    version_output="`${RUSTC} --version`"

    AC_MSG_CHECKING(the rust version supported by ${RUSTC})
    
    read rustc_name rust_version <<< ${version_output}

    AC_MSG_RESULT(${rust_version})

    case $rust_version in
        *.*.*-beta.* )
            rust_channel=beta
            IFS="." read rust_major rust_minor _lcrust_rest <<< ${rust_version}
            IFS="-" read rust_patch <<< ${_lcrust_rest}
            ;;
        *.*.*-* )
            IFS="." read rust_major rust_minor _lcrust_rest <<< ${rust_version}
            IFS="-" read rust_patch rust_channel <<< ${_lcrust_rest}
            ;;
        *.*.* )
            rust_channel=stable
            IFS="." read rust_major rust_minor rust_patch <<< ${rust_version}
            ;;
    esac
    AC_MSG_CHECKING(whether $RUSTC is lccc)
    case $rustc_name in
        lcrust* | lccc* ) dnl lccc doesn't distinguish between stable and unstable compiler, 
            rustc_is_lccc=yes
            ;;
        * )
            rustc_is_lccc=no
            ;;
    esac
    AC_MSG_RESULT([$rustc_is_lccc])
    
    AC_SUBST(rustc_name)
    AC_SUBST(rust_version)
    AC_SUBST(rust_channel)
    AC_SUBST(rust_major)
    AC_SUBST(rust_minor)
    AC_SUBST(rust_patch)
])

AC_DEFUN([LCRUST_PROG_RUSTC_FOR_BUILD],[
    AC_REQUIRE([AX_PROG_CC_FOR_BUILD])
    AC_REQUIRE([AC_CANONICAL_BUILD])
    AC_ARG_VAR(RUSTC_FOR_BUILD,[Rust compiler to use on the build system])
    AC_ARG_VAR(RUSTFLAGS_FOR_BUILD,[Flags to pass to the rust compiler for the build system])

    AC_MSG_NOTICE([checking for the compiler to use for $build...])

    AC_PATH_PROGS(RUSTC_FOR_BUILD,[rustc lcrustc $build-gccrs gccrs])

    if test "$RUSTC_FOR_BUILD" \= ""
    then
        AC_MSG_NOTICE([checking for the compiler to use for $build... not found])
        AC_MSG_ERROR([Failed to find a rust compiler. Install rustc in PATH, or set RUSTC_FOR_BUILD to a suitable compiler])
    fi

    AC_MSG_NOTICE([checking for the compiler to use for $build... $RUSTC_FOR_BUILD])

   AC_MSG_CHECKING([how to compile for $build with $RUSTC_FOR_BUILD])
    case x$RUSTC_FOR_BUILD in 
        x${build_alias}-* | x*[\\/]${build_alias}-* )
            rustc_build_target=${build_alias}
            AC_MSG_RESULT([Not needed])
            ;;
        x${build}-* | x*[\\/]${build}-* )
            rustc_build_target=${build}
            AC_MSG_RESULT([Not needed])
            ;;

        x* )
            SAVE_RUSTFLAGS_FOR_BUILD="$RUSTFLAGS_FOR_BUILD"
            if test x$build_alias != x 
            then
                RUSTFLAGS_FOR_BUILD="$RUSTFLAGS_FOR_BUILD --target $build_alias"
                echo 'fn main(){}' > comptest.rs 
                echo Trying target $build_alias >> config.log
                echo "$RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                $RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                if test $? -eq 0
                then
                    rustc_host_target=$build_alias
                else
                    echo Using target $build_alias failed >> config.log
                fi 
            fi
            
            if test x$rustc_build_target \= x
            then 
                RUSTFLAGS_FOR_BUILD="$SAVE_RUSTFLAGS_FOR_BUILD --target $build"
                echo 'fn main(){}' > comptest.rs 
                echo Trying target $build >> config.log
                echo "$RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                $RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                if test $? -eq 0
                then
                    rustc_host_target=$build
                else
                    echo Using target $build failed >> config.log
                fi 
            fi

            if test x$rustc_build_target \= x
            then
                case $build in                
                x86_64-*-mingw32 | i*86-*-mingw32 )
                    IFS="-" read rustc_host_arch rustc_host_vendor rustc_host_sys <<< "$host"
                    RUSTFLAGS="$SAVE_RUSTFLAGS --target ${rustc_host_arch}-pc-windows-gnu"
                    echo '#![no_std]' > comptest.rs 
                    echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                    $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_host_target=${rustc_host_arch}-pc-windows-gnu
                    fi
                    ;;
                x86_64-*-*-* | i*86-*-*-* )
                    IFS="-" read rustc_build_arch rustc_build_vendor rustc_build_os rustc_build_env <<< "$build"
                    RUSTFLAGS_FOR_BUILD="$SAVE_RUSTFLAGS_FOR_BUILD --target ${rustc_build_arch}-unknown-${rustc_build_os}-${rustc_build_env}"
                    echo 'fn main(){}' > comptest.rs 
                    echo "$RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                $RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_build_target=${rustc_build_arch}-unknown-${rustc_build_os}-${rustc_build_env}
                    fi 
                    ;;
                x86_64-*-* | i*86-*-* )
                    IFS="-" read rustc_build_arch rustc_build_vendor rustc_build_sys <<< "$build"
                    RUSTFLAGS_FOR_BUILD="$SAVE_RUSTFLAGS_FOR_BUILD --target ${rustc_build_arch}-unknown-${rustc_build_sys}"
                    echo 'fn main(){}' > comptest.rs 
                    echo "$RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                    $RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_host_target=${rustc_build_arch}-unknown-${rustc_build_sys}
                    fi 
                    ;;

                *-*-* )
                    IFS="-" read rustc_host_arch rustc_host_vendor rustc_host_sys <<< "$host"
                    RUSTFLAGS="$SAVE_RUSTFLAGS --target ${rustc_host_arch}-${rustc_host_sys}"
                    echo 'fn main(){}' > comptest.rs 

                    echo "$RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs" >> config.log
                    $RUSTC $RUSTFLAGS --crate-type rlib --crate-name comptest comptest.rs 2>> config.log > /dev/null

                    if test $? -eq 0
                    then
                        rustc_host_target=${rustc_host_arch}-${rustc_host_sys}
                    fi
                    ;;
                esac
            fi
            if test x$rustc_build_target \= x
            then
                AC_MSG_RESULT([not found])
                AC_MSG_ERROR([Cannot compile to $build with $RUSTC])
            else
                AC_MSG_RESULT([--target $rustc_build_target])
            fi
        ;;
    esac

    rm -f comptest.rs libcomptest.rlib
    AC_MSG_CHECKING([whether Rust compiler works])
    echo 'fn main(){}' > test.rs 
    $RUSTC_FOR_BUILD $RUSTFLAGS_FOR_BUILD --crate-type bin --crate-name test test.rs 2>> config.log > /dev/null
    if test $? -ne 0
    then
        AC_MSG_RESULT([no])
        AC_MSG_ERROR([Cannot compile a simple program with $RUSTC_FOR_BUILD])
    fi
    
    ./test${EXEEXT_FOR_BUILD}
    if test $? -ne 0
    then
        AC_MSG_RESULT([no])
        AC_MSG_ERROR([Cannot run executables compiled by $RUSTC_FOR_BUILD])
    fi

    rm -rf test.rs test${EXEEXT_FOR_BUILD}

    AC_MSG_RESULT([yes])

    AC_SUBST(RUSTC_FOR_BUILD)
    AC_SUBST(RUSTFLAGS_FOR_BUILD)
])

AC_DEFUN([LCRUST_RUSTC_VERSION_FOR_BUILD],[
    AC_REQUIRE([LCRUST_PROG_RUSTC_FOR_BUILD])

    version_output="`${RUSTC_FOR_BUILD} --version`"

    AC_MSG_CHECKING(the rust version supported by ${RUSTC_FOR_BUILD})
    
    read build_rustc_name build_rust_version <<< ${version_output}

    AC_MSG_RESULT(${build_rust_version})

    case $build_rust_version in
        *.*.*-beta.* )
            rust_channel=beta
            IFS="." read build_rust_major build_rust_minor _lcrust_rest <<< ${build_rust_version}
            IFS="-" read build_rust_patch <<< ${_lcrust_rest}
            ;;
        *.*.*-* )
            IFS="." read build_rust_major build_rust_minor _lcrust_rest <<< ${build_rust_version}
            IFS="-" read build_rust_patch build_rust_channel <<< ${_lcrust_rest}
            ;;
        *.*.* )
            rust_channel=stable
            IFS="." read build_rust_major build_rust_minor build_rust_patch <<< ${build_rust_version}
            ;;
    esac
    AC_MSG_CHECKING(whether $RUSTC_FOR_BUILD is lccc)
    case $build_rustc_name in
        lcrust* | lccc* ) dnl lccc doesn't distinguish between stable and unstable compiler, 
            build_rustc_is_lccc=yes
            ;;
        * )
            build_rustc_is_lccc=no
            ;;
    esac
    AC_MSG_RESULT([$build_rustc_is_lccc])
    
    AC_SUBST(build_rustc_name)
    AC_SUBST(build_rust_version)
    AC_SUBST(build_rust_channel)
    AC_SUBST(build_rust_major)
    AC_SUBST(build_rust_minor)
    AC_SUBST(build_rust_patch)
])


AC_DEFUN([LCRUST_TRY_COMPILE],[
    echo ["$1"] >> test.rs
    ${RUSTC} ${RUSTFLAGS} --crate-type rlib --crate-name test --emit link=libtest.rlib test.rs

    if test $? -eq 0 
    then
        rm -f test.rs libtest.rlib
        $2
    else
        rm -f test.rs libtest.rlib
        $3
    fi
])

AC_DEFUN([LCRUST_TRY_COMPILE_FOR_BUILD],[
    echo ["$1"] >> test.rs
    ${RUSTC_FOR_BUILD} ${RUSTFLAGS_FOR_BUILD} --crate-type rlib --crate-name test --emit link=libtest.rlib test.rs

    if test $? -eq 0 
    then
        rm -f test.rs libtest.rlib
        try_compile_result=yes
        $2
    else
        rm -f test.rs libtest.rlib
        try_compile_result=no
        $3
    fi
])
