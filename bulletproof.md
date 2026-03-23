# Bulletproof
want to prove <a, b> in a zk way

G and H are known basis vectors, B and G' are known points
commit 5 values to the verifier:

C = <a, G> + <b, H> + g1 * B
L = <sa, G> + <sb, H> + g2 * B
C_t = <a, b>G' + g3 * B
L_t = (<a, sb> + <b, sa>)G' + g4 * B
Q_t = <sa, sb>G' + g5 * B

verifier sends u

let
𝛑\_1 = g1 + g2 * u -> prover sent
𝛑\_2 = g3 + g4 * u + g5 * u^2 -> prover sent
V_lr = C + L * u
V_t = C_t + L_t * u + Q_t * u^2
t(u) = <a + sa * u, b + sb * u>G' -> prover sent
A_u = <a + sa * u, G>
B_u = <b + sb * u, H>

verifier wants to check
1. V_lr = A_u + B_u + 𝛑\_1 * B
which checks that our polynomials representing a and b are valid

2. V_t = t(u) + 𝛑\_2 * B
which checks our committed multiplication polynomial

3. t(u) = <A_u\[0\], B_u\[0\]>
which checks if our polynomials hold their relation as we specified (t(x) = l(x)r(x))

checks 1 and 2 can be done with O(1) communication

for check 3 we could send the vectors A_u\[0\] = a + sa*u, B_u\[0\] = b + sb * u
but it would take O(n) communication

We know a recursive argument to prove P = <a,  G> in logn communication

applying P = <A_u\[0\] + B_u\[0\] + \[t(u)\], A_u\[1\] + B_u\[1\] + \[G'\]>
where + there is vector concatenation

would get us to where we want to be, but that would still require communicating the vectors.
The key observation is that you can distribute the dot product into P = A_u + B_u + t(u)G'.

This means we can apply the O(logn) argument to P with O(1) communication.

If we were to know P, our last verification to imply that 3. is true would be:
P - V_lr + 𝛑\_1 * B = t(u) * G'.

Thus we can run the argument with P = V_lr + t(u) * G' - 𝛑\_1 * B

# IPA <a, G>
How do you prove <a, G> in O(logn) communication?

-> doesn't have to be zero knowledge, our vectors are already hiding.

ipa (P, N, G) verifier pov, + (a) for prover pov
N is power of 2.

N = 1:
send a, verifier checks a * G = P

N > 1:
You want to compute <a, G>.

Break a and g apart with sequential pairs, i.e. a = \[a1, a2, a3, a4\] => \[a1 + a2, a3 + a4\] = a'

Initially we can think of the inner product of these two vectors as:

P' = <a', G'> = <a_i + a_(i+1), G_i + G_(i+1)> = <a, G> + <a_i, G_(i+1)> + <a_(i+1), G_i>
                                                   P    +      L         +         R

Now, the problem is that the prover needs to commit to these values, otherwise he could just manipulate the equation as he sees fit.

prover: Compute P', L, R and send to verifier

verifier: sends u

Both the prover and verifier compute G' = \[G_i + G_(i+1) * u\]
prover computes a' = \[a_i + a_(i+1) * u ^ -1\]

then our final multiplication becomes
P' = <a', G'>
   = <a_i + a_(i+1) * u ^ -1, G_i + G_(i+1) * u>
   = P + <a_i, G_(i+1) * u> + <a_(i+1) * u^-1, G_i>
   = P + L * u + R * u ^-1

P' is valid <- ipa(P', N/2, G') + (a')

the ipa procedure proves that the L, R commitments are validated, since otherwise
the base case would fail, since P' is computed separately by both prover and verifier.

--- optimizing public computation of G'
lets go over G if N = 8:

step 0:
\[g1, g2, g3, g4, g5, g6, g7, g8\] - 8 0 bit numbers

step 1:
\[g1 + g2 * x0, g3 + g4 * x0, g5 + g6 * x0, g7 + g8 * x0\] - 4 1 bit numbers

step 2:
\[g1 + g2 * x0 + g3 * x1 + g4 * x1 * x0, g5 + g6 * x0 + g7 * x1 + g8 * x1 * x0\] - 2 2 bit numbers

step 3:
\[g1 + g2 * x0 + g3 * x1 + g4 * x1 * x0 + g5 * x2 + g6 * x2 * x0 + g7 * x2 * x1 + g8 * x2 * x1 * x0\] - 1 3 bit number

basically the scalar at the end becomes the mask of i multiplied by the challenge at each bit.
we can compute this recursively by simply starting with \[1\] and at each challenge duplicating the array via append(current, current * challenge)

\[1\] => \[1, x0\] => \[1, x0, x1, x0 * x1\] => \[1, x0, x1, x0 * x1, x2, x2 * x0, x2 * x1, x2 * x1 * x0\].

This can be proved via induction (omitted)
