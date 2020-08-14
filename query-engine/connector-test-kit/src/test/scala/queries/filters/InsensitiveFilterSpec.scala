package queries.filters

import org.scalatest.{FlatSpec, Matchers}
import util.ConnectorCapability.JoinRelationLinksCapability
import util._

class InsensitiveFilterSpec extends FlatSpec with Matchers with ApiSpecBase {
  val project: Project = ProjectDsl.fromString {
    """
      |model TestModel {
      |  id  String @id @default(cuid())
      |  str String
      |}
    """.stripMargin
  }

  override protected def beforeAll(): Unit = {
    super.beforeAll()
    database.setup(project)
  }

  def create(str: String): String = {
    val res = server.query(
      s"""mutation {
         |  createOneTestModel(
         |    data: {
         |      str: "$str"
         |    })
         |  {
         |    id
         |  }
         |}
      """.stripMargin,
      project,
      legacy = false
    )

    res.pathAsString("data.createOneTestModel.id")
  }

  "Case insensitive filters" should "work with string matchers" taggedAs (IgnoreSQLite, IgnoreMongo, IgnoreMySql) in {
    create("a test")
    create("A Test")
    create("b test")

    var res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    startsWith: "a",
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be("""{"data":{"findManyTestModel":[{"str":"a test"},{"str":"A Test"}]}}""")

    res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    endsWith: "Test",
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be("""{"data":{"findManyTestModel":[{"str":"a test"},{"str":"A Test"},{"str":"b test"}]}}""")

    res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    contains: "Te",
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be("""{"data":{"findManyTestModel":[{"str":"a test"},{"str":"A Test"},{"str":"b test"}]}}""")
  }

  "Case insensitive filters" should "work with negated string matchers" taggedAs (IgnoreSQLite, IgnoreMongo, IgnoreMySql) in {
    create("a test")
    create("A Test")
    create("b test")

    var res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    not: { startsWith: "a" }
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be("""{"data":{"findManyTestModel":[{"str":"b test"}]}}""")

    res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    not: { endsWith: "Test" }
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be("""{"data":{"findManyTestModel":[]}}""")

    res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    not: { contains: "Te" },
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be("""{"data":{"findManyTestModel":[]}}""")
  }

  "Case insensitive filters" should "work with comparator operations" taggedAs (IgnoreSQLite, IgnoreMongo, IgnoreMySql) in {
    // Note: Postgres collations order characters differently than, say, using .sort in most programming languages,
    // which is why the results of <, >, etc. are non-obvious at a glance.
    create("A")
    create("æ")
    create("Æ")
    create("bar")
    create("aÆB")
    create("AÆB")
    create("aæB")
    create("aB")

    var res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    equals: "æ"
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )
    res.toString() should be("""{"data":{"findManyTestModel":[{"str":"æ"},{"str":"Æ"}]}}""")

    res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    gte: "aÆB"
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be(
      """{"data":{"findManyTestModel":[{"str":"æ"},{"str":"Æ"},{"str":"bar"},{"str":"aÆB"},{"str":"AÆB"},{"str":"aæB"},{"str":"aB"}]}}""")

    res = server.query(
      """{
        |findManyTestModel(where: {
        |  str: {
        |    lt: "aÆB"
        |    mode: insensitive
        |  }
        |}) {
        |  str
        |}}
      """.stripMargin,
      project,
      legacy = false
    )

    res.toString() should be("""{"data":{"findManyTestModel":[{"str":"A"}]}}""")
  }
}
