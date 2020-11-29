extern putchard(char);

def printdensity(d)
  if d > 8 then
    putchard(32)
  else if d > 4 then
    putchard(46)
  else if d > 2 then
    putchard(43)
  else
    putchard(42);

def mandelconverger(real imag iters creal cimag)
  if iters > 255 | (real*real + imag*imag > 4) then
    iters
  else
    mandelconverger( real*real - imag*imag + creal
                   , 2*real*imag + cimag
                   , iters+1
                   , creal
                   , cimag
                   );

def mandelconverge(real imag)
  mandelconverger(real, imag, 0, real, imag);

def mandelhelp(xmin xmax xstep   ymin ymax ystep)
  for y = ymin, y < ymax, ystep in
  (
    (for x = xmin, x < xmax, xstep in printdensity(mandelconverge(x,y))) : putchard(10)
  );

def mandel(realstart imagstart realmag imagmag)
  mandelhelp(realstart, realstart+realmag*78, realmag, imagstart, imagstart+imagmag*40, imagmag);

def main()
  mandel(-2.3, -1.3, 0.05, 0.07);
