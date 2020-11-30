# Rust Kaleidoscope

This is an implementation of the [Kaleidoscope toy language](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html)-to-LLVM-IR JIT compiler in Rust.

I wrote this in November of 2020 as a way to play with Rust, Nom, and LLVM / JIT compilation.

One of my goals was to *not* use Nom's macros, which results in the code being a little more verbose than it would otherwise be.

I got a lot of inspiration from other projects:

1. Stephen Diehl's [Haskell Kaleidoscope tutorial](https://github.com/sdiehl/kaleidoscope), where I got the idea to do this in Rust, and first learned of Kaleidoscope.
2. The [inkwell](https://github.com/TheDan64/inkwell) [implementation of Kaleidoscope](https://github.com/TheDan64/inkwell/blob/master/examples/kaleidoscope/main.rs), which I discovered after writing my parser, and got of inspiration and maybe some code snippets.
3. The [iron-kaleidoscope](https://github.com/jauhien/iron-kaleidoscope) project is a full tutorial for implementing a Kaleidoscope JIT in Rust. I didn't look at the project, but it looks much better than my implementation here.

## Usage

1. Use [`llvmenv`](https://github.com/termoshtt/llvmenv) to install llvm 10.0
2. Use `cargo run examples/mandelbrot.ks` to compile the program and run the Mandelbrot example.

## Mandelbrot output

The output of running mandelbrot.ks, which outputs the parsed tree, LLVM IR and then the output of the code:

```
$ cargo run examples/mandelbrot.ks

Parsed: [Extern("putchard", ["char"]), Function("printdensity", ["d"], IfExpr(BinOp(GreaterThan, Var("d"), Float(8.0)), Call("putchard", [Float(32.0)]), IfExpr(BinOp(GreaterThan, Var("d"), Float(4.0)), Call("putchard", [Float(46.0)]), IfExpr(BinOp(GreaterThan, Var("d"), Float(2.0)), Call("putchard", [Float(43.0)]), Call("putchard", [Float(42.0)]))))), Function("unary!", ["v"], IfExpr(Var("v"), Float(0.0), Float(1.0))), Function("unary-", ["v"], BinOp(Minus, Float(0.0), Var("v"))), Function("binary|", ["LHS", "RHS"], IfExpr(Var("LHS"), Float(1.0), IfExpr(Var("RHS"), Float(1.0), Float(0.0)))), Function("binary&", ["LHS", "RHS"], IfExpr(Call("unary!", [Var("LHS")]), Float(0.0), Call("unary!", [Call("unary!", [Var("RHS")])]))), Function("binary:", ["x", "y"], Var("y")), Function("mandelconverger", ["real", "imag", "iters", "creal", "cimag"], IfExpr(Call("binary|", [BinOp(GreaterThan, Var("iters"), Float(255.0)), BinOp(GreaterThan, BinOp(Plus, BinOp(Multiply, Var("real"), Var("real")), BinOp(Multiply, Var("imag"), Var("imag"))), Float(4.0))]), Var("iters"), Call("mandelconverger", [BinOp(Plus, BinOp(Minus, BinOp(Multiply, Var("real"), Var("real")), BinOp(Multiply, Var("imag"), Var("imag"))), Var("creal")), BinOp(Plus, BinOp(Multiply, BinOp(Multiply, Float(2.0), Var("real")), Var("imag")), Var("cimag")), BinOp(Plus, Var("iters"), Float(1.0)), Var("creal"), Var("cimag")]))), Function("mandelconverge", ["real", "imag"], Call("mandelconverger", [Var("real"), Var("imag"), Float(0.0), Var("real"), Var("imag")])), Function("mandelhelp", ["xmin", "xmax", "xstep", "ymin", "ymax", "ystep"], ForInExpr("y", Var("ymin"), BinOp(LessThan, Var("y"), Var("ymax")), Var("ystep"), Call("binary:", [ForInExpr("x", Var("xmin"), BinOp(LessThan, Var("x"), Var("xmax")), Var("xstep"), Call("printdensity", [Call("mandelconverge", [Var("x"), Var("y")])])), Call("putchard", [Float(10.0)])]))), Function("mandel", ["realstart", "imagstart", "realmag", "imagmag"], Call("mandelhelp", [Var("realstart"), BinOp(Plus, Var("realstart"), BinOp(Multiply, Var("realmag"), Float(78.0))), Var("realmag"), Var("imagstart"), BinOp(Plus, Var("imagstart"), BinOp(Multiply, Var("imagmag"), Float(40.0))), Var("imagmag")])), Function("main", [], Call("mandel", [Float(-2.3), Float(-1.3), Float(0.05), Float(0.07)]))]

; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare double @putchard(double %char)

define double @printdensity(double %d) {
entry:
  %tmpcmp = fcmp ugt double %d, 8.000000e+00
  br i1 %tmpcmp, label %then, label %else

then:                                             ; preds = %entry
  %tmp = call double @putchard(double 3.200000e+01)
  br label %ifcont

else:                                             ; preds = %entry
  %tmpcmp4 = fcmp ugt double %d, 4.000000e+00
  br i1 %tmpcmp4, label %then7, label %else8

ifcont:                                           ; preds = %then15, %else16, %then7, %then
  %iftmp21 = phi double [ %tmp, %then ], [ %tmp10, %then7 ], [ %tmp18, %then15 ], [ %tmp19, %else16 ]
  ret double %iftmp21

then7:                                            ; preds = %else
  %tmp10 = call double @putchard(double 4.600000e+01)
  br label %ifcont

else8:                                            ; preds = %else
  %tmpcmp12 = fcmp ugt double %d, 2.000000e+00
  br i1 %tmpcmp12, label %then15, label %else16

then15:                                           ; preds = %else8
  %tmp18 = call double @putchard(double 4.300000e+01)
  br label %ifcont

else16:                                           ; preds = %else8
  %tmp19 = call double @putchard(double 4.200000e+01)
  br label %ifcont
}

define double @"unary!"(double %v) {
entry:
  %ifcond = fcmp ueq double %v, 0.000000e+00
  %iftmp = select i1 %ifcond, double 1.000000e+00, double 0.000000e+00
  ret double %iftmp
}

define double @unary-(double %v) {
entry:
  %tmpsub = fsub double 0.000000e+00, %v
  ret double %tmpsub
}

define double @"binary|"(double %LHS, double %RHS) {
entry:
  %ifcond = fcmp ueq double %LHS, 0.000000e+00
  %ifcond5 = fcmp ueq double %RHS, 0.000000e+00
  %0 = and i1 %ifcond, %ifcond5
  %iftmp9 = select i1 %0, double 0.000000e+00, double 1.000000e+00
  ret double %iftmp9
}

define double @"binary&"(double %LHS, double %RHS) {
entry:
  %tmp = call double @"unary!"(double %LHS)
  %ifcond = fcmp ueq double %tmp, 0.000000e+00
  br i1 %ifcond, label %else, label %ifcont

else:                                             ; preds = %entry
  %tmp5 = call double @"unary!"(double %RHS)
  %tmp6 = call double @"unary!"(double %tmp5)
  br label %ifcont

ifcont:                                           ; preds = %entry, %else
  %iftmp = phi double [ %tmp6, %else ], [ 0.000000e+00, %entry ]
  ret double %iftmp
}

define double @"binary:"(double %x, double %y) {
entry:
  ret double %y
}

define double @mandelconverger(double %real, double %imag, double %iters, double %creal, double %cimag) {
entry:
  %tmpcmp = fcmp ugt double %iters, 2.550000e+02
  %tmpbool = uitofp i1 %tmpcmp to double
  %tmpmul = fmul double %real, %real
  %tmpmul11 = fmul double %imag, %imag
  %tmpadd = fadd double %tmpmul, %tmpmul11
  %tmpcmp12 = fcmp ugt double %tmpadd, 4.000000e+00
  %tmpbool13 = uitofp i1 %tmpcmp12 to double
  %tmp = call double @"binary|"(double %tmpbool, double %tmpbool13)
  %ifcond = fcmp ueq double %tmp, 0.000000e+00
  br i1 %ifcond, label %else, label %ifcont

else:                                             ; preds = %entry
  %tmpsub = fsub double %tmpmul, %tmpmul11
  %tmpadd22 = fadd double %tmpsub, %creal
  %tmpmul24 = fmul double %real, 2.000000e+00
  %tmpmul26 = fmul double %tmpmul24, %imag
  %tmpadd28 = fadd double %tmpmul26, %cimag
  %tmpadd30 = fadd double %iters, 1.000000e+00
  %tmp33 = call double @mandelconverger(double %tmpadd22, double %tmpadd28, double %tmpadd30, double %creal, double %cimag)
  br label %ifcont

ifcont:                                           ; preds = %entry, %else
  %iftmp = phi double [ %tmp33, %else ], [ %iters, %entry ]
  ret double %iftmp
}

define double @mandelconverge(double %real, double %imag) {
entry:
  %tmp = call double @mandelconverger(double %real, double %imag, double 0.000000e+00, double %real, double %imag)
  ret double %tmp
}

define double @mandelhelp(double %xmin, double %xmax, double %xstep, double %ymin, double %ymax, double %ystep) {
entry:
  br label %loop

loop:                                             ; preds = %afterloop, %entry
  %y20 = phi double [ %nextvar25, %afterloop ], [ %ymin, %entry ]
  br label %loop9

loop9:                                            ; preds = %loop9, %loop
  %x16 = phi double [ %nextvar, %loop9 ], [ %xmin, %loop ]
  %tmp = call double @mandelconverge(double %x16, double %y20)
  %tmp12 = call double @printdensity(double %tmp)
  %tmpcmp = fcmp ult double %x16, %xmax
  %nextvar = fadd double %xstep, %x16
  br i1 %tmpcmp, label %loop9, label %afterloop

afterloop:                                        ; preds = %loop9
  %tmp17 = call double @putchard(double 1.000000e+01)
  %tmp18 = call double @"binary:"(double 0.000000e+00, double %tmp17)
  %tmpcmp22 = fcmp ult double %y20, %ymax
  %nextvar25 = fadd double %ystep, %y20
  br i1 %tmpcmp22, label %loop, label %afterloop27

afterloop27:                                      ; preds = %afterloop
  ret double 0.000000e+00
}

define double @mandel(double %realstart, double %imagstart, double %realmag, double %imagmag) {
entry:
  %tmpmul = fmul double %realmag, 7.800000e+01
  %tmpadd = fadd double %realstart, %tmpmul
  %tmpmul12 = fmul double %imagmag, 4.000000e+01
  %tmpadd13 = fadd double %imagstart, %tmpmul12
  %tmp = call double @mandelhelp(double %realstart, double %tmpadd, double %realmag, double %imagstart, double %tmpadd13, double %imagmag)
  ret double %tmp
}

define double @main() {
entry:
  %tmp = call double @mandel(double -2.300000e+00, double -1.300000e+00, double 5.000000e-02, double 7.000000e-02)
  ret double %tmp
}

*******************************************************************************
*******************************************************************************
****************************************++++++*********************************
************************************+++++...++++++*****************************
*********************************++++++++.. ...+++++***************************
*******************************++++++++++..   ..+++++**************************
******************************++++++++++.     ..++++++*************************
****************************+++++++++....      ..++++++************************
**************************++++++++.......      .....++++***********************
*************************++++++++.   .            ... .++**********************
***********************++++++++...                     ++**********************
*********************+++++++++....                    .+++*********************
******************+++..+++++....                      ..+++********************
**************++++++. ..........                        +++********************
***********++++++++..        ..                         .++********************
*********++++++++++...                                 .++++*******************
********++++++++++..                                   .++++*******************
*******++++++.....                                    ..++++*******************
*******+........                                     ...++++*******************
*******+... ....                                     ...++++*******************
*******+++++......                                    ..++++*******************
*******++++++++++...                                   .++++*******************
*********++++++++++...                                  ++++*******************
**********+++++++++..        ..                        ..++********************
*************++++++.. ..........                        +++********************
******************+++...+++.....                      ..+++********************
*********************+++++++++....                    ..++*********************
***********************++++++++...                     +++*********************
*************************+++++++..   .            ... .++**********************
**************************++++++++.......      ......+++***********************
****************************+++++++++....      ..++++++************************
*****************************++++++++++..     ..++++++*************************
*******************************++++++++++..  ...+++++**************************
*********************************++++++++.. ...+++++***************************
***********************************++++++....+++++*****************************
***************************************++++++++********************************
*******************************************************************************
*******************************************************************************
*******************************************************************************
*******************************************************************************
*******************************************************************************
```
