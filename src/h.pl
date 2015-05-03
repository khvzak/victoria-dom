use v5.16;
use strict;
use warnings;
use utf8;

my @x = qw(
            form frameset h1 h2 h3 h4 h5 h6 head
        header hgroup html i iframe li listing main marquee menu nav nobr
        noembed noframes noscript object ol optgroup option p plaintext pre rp
        rt s script section select small strike strong style summary table
        tbody td template textarea tfoot th thead title tr tt u ul xmp
);

say join(", ", map { '"'.$_.'"' } @x);
