#include "Laplacian.h"
#include "Parameters.h"
#include "PointwiseOps.h"
#include "Reductions.h"
#include "Utilities.h"

#include <iostream>

void ConjugateGradients(
    float (&x)[XDIM][YDIM][ZDIM],
    const float (&f)[XDIM][YDIM][ZDIM],
    float (&p)[XDIM][YDIM][ZDIM],
    float (&r)[XDIM][YDIM][ZDIM],
    float (&z)[XDIM][YDIM][ZDIM],
    const bool writeIterations)
{
    // Algorithm : Line 2
    ComputeLaplacian(x, z, 2);
    Saxpy(z, f, r, -1, 2);
    float nu = Norm(r, 2);

    // Algorithm : Line 3
    if (nu < nuMax) return;
        
    // Algorithm : Line 4
    Copy(r, p, 4);
    float rho=InnerProduct(p, r, 4);
        
    // Beginning of loop from Line 5
    for(int k=0;;k++)
    {
        std::cout << "Residual norm (nu) after " << k << " iterations = " << nu << std::endl;

        // Algorithm : Line 6
        ComputeLaplacian(p, z, 6);
        float sigma=InnerProduct(p, z, 6);

        // Algorithm : Line 7
        float alpha=rho/sigma;

        // Algorithm : Line 8
        Saxpy(z, r, r, -alpha, 8);
        nu=Norm(r, 8);

        // Algorithm : Lines 9-12
        if (nu < nuMax || k == kMax) {
            Saxpy(p, x, x, alpha, 10);
            std::cout << "Conjugate Gradients terminated after " << k << " iterations; residual norm (nu) = " << nu << std::endl;
            if (writeIterations) WriteAsImage("x", x, k, 0, 127);
            return;
        }
            
        // Algorithm : Line 13
        Copy(r, z, 13);
        float rho_new = InnerProduct(z, r, 13);

        // Algorithm : Line 14
        float beta = rho_new/rho;

        // Algorithm : Line 15
        rho=rho_new;

        // Algorithm : Line 16
        Saxpy(p, x, x, alpha, 16, "1st");
        Saxpy(p, r, p, beta, 16, "2nd");

        if (writeIterations) WriteAsImage("x", x, k, 0, 127);
    }
}
