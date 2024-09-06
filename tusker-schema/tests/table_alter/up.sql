ALTER TABLE "public"."a"
    ALTER COLUMN "id" bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    ALTER COLUMN "name" character varying(50) NOT NULL;

ALTER TABLE "public"."a" ADD CONSTRAINT "a_age_check" CHECK ((age >= 0));

ALTER TABLE "public"."a" ADD CONSTRAINT "a_name_check" CHECK (((name)::text <> ''::text));

ALTER TABLE "public"."a" ADD CONSTRAINT "a_pkey" PRIMARY KEY (id);
